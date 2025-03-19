// src/main.rs
mod analyze_sentence;
mod call_ollama;
mod cli;
mod endpoint_client;
mod grpc_server;
mod json_helper;
mod models;
mod prompts;
mod sentence_service;

use std::sync::Arc;
mod workflow;
use crate::models::config::load_models_config;
use crate::models::providers::{create_provider, ModelProvider, ProviderConfig};
use cli::ProviderType;

use clap::Parser;
use cli::{handle_cli, Cli};
use dotenv::dotenv;
use endpoint_client::get_default_api_url;
use grpc_logger::load_config;
use grpc_server::start_sentence_grpc_server;
use std::env;
use std::error::Error;
use tokio::signal;
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

#[derive(Clone)]
pub struct AppState {
    //provider: Arc<dyn ModelProvider>,
    //log_config: Arc<LogConfig>,
    //api_url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let _log_config = load_config("config.yaml")?;

    Registry::default()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("INFO")))
        .init();

    // Parse CLI arguments
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // Enhance error message for missing provider
            if e.to_string()
                .contains("required arguments were not provided")
                && e.to_string().contains("--provider")
            {
                eprintln!("ERROR: You must specify which provider to use!");
                eprintln!("Options: --provider ollama   (for local Ollama instance)");
                eprintln!("         --provider claude   (for Claude API, requires API key)");
                eprintln!("\nExample: cargo run -- --provider ollama");
                std::process::exit(1);
            } else {
                // Display the original error
                e.exit();
            }
        }
    };

    // Load model configuration
    let _models_config = load_models_config().await?;

    // Initialize provider based on CLI provider type
    let use_claude = matches!(cli.provider, ProviderType::Claude);

    // Create the provider
    let provider: Box<dyn ModelProvider> = if use_claude {
        // Load .env file and check for Claude API key
        dotenv().ok();
        match env::var("CLAUDE_API_KEY") {
            Ok(api_key) => {
                info!("Using Claude API");
                let config = ProviderConfig {
                    enabled: true,
                    api_key: Some(api_key),
                    host: None,
                };
                create_provider(&config).expect("Failed to create Claude provider")
            }
            Err(_) => {
                error!("Claude API key not found in .env file. Please add CLAUDE_API_KEY to .env");
                std::process::exit(1);
            }
        }
    } else {
        // Use self-hosted Ollama
        info!("Using self-hosted Ollama");
        let config = ProviderConfig {
            enabled: true,
            host: Some("http://localhost:11434".to_string()),
            api_key: None,
        };
        create_provider(&config).expect("Failed to create Ollama provider")
    };

    // Wrap the provider in an Arc so we can clone it
    let provider_arc: Arc<dyn ModelProvider> = Arc::from(provider);

    // Get API URL from CLI or config
    let api_url = if let Some(url) = cli.api.clone() {
        Some(url)
    } else {
        // Only get default if we're not in CLI prompt mode
        if cli.prompt.is_none() {
            match get_default_api_url().await {
                Ok(url) => Some(url),
                Err(e) => {
                    error!("Failed to get default API URL: {}", e);
                    None
                }
            }
        } else {
            None
        }
    };

    //let app_state = AppState {
    //    provider: provider_arc.clone(),
    //    log_config: Arc::new(log_config),
    //    api_url: api_url.clone(),
    //};

    // Handle CLI command if present, otherwise start gRPC server
    match cli.prompt {
        Some(_) => {
            handle_cli(cli, provider_arc).await?;
        }
        None => {
            let provider_name = match cli.provider {
                ProviderType::Claude => "Claude API",
                ProviderType::Ollama => "Ollama self-hosted models",
            };
            info!(
                "No prompt provided, starting gRPC server with {}...",
                provider_name
            );

            // Start the gRPC server with our API URL if provided
            let grpc_server = tokio::spawn(async move {
                if let Err(e) =
                    start_sentence_grpc_server(provider_arc.clone(), api_url, cli.email).await
                {
                    error!("gRPC server error: {:?}", e);
                }
            });

            info!("Semantic server started");

            // Wait for CTRL-C
            tokio::select! {
                _ = signal::ctrl_c() => {
                    info!("Received shutdown signal, initiating graceful shutdown...");
                }
                result = grpc_server => {
                    if let Err(e) = result {
                        error!("gRPC server task error: {:?}", e);
                    }
                }
            }

            info!("Server shutting down");
        }
    }

    Ok(())
}
