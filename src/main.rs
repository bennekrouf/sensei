mod analyze_sentence;
mod call_ollama;
mod cli;
mod grpc_server;
mod json_helper;
mod models;
mod prompts;
mod sentence_service;
use std::sync::Arc;
mod workflow;
use crate::models::providers::ProviderSelector;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};
use clap::Parser;
use cli::{handle_cli, Cli};
use grpc_logger::load_config;
use grpc_logger::setup_logging;
use grpc_logger::LogConfig;
use grpc_server::start_sentence_grpc_server;
use models::providers::init_environment;
use models::providers::ModelProvider;
use std::error::Error;
use tokio::signal;
use tracing::{error, info};

#[derive(Clone)]
pub struct AppState {
    provider: Arc<dyn ModelProvider>,
    log_config: Arc<LogConfig>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let log_config = load_config("config.yaml")?;

    Registry::default()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("INFO")))
        .init();
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize provider based on CLI flag
    let provider = ProviderSelector::select_provider(cli.claude);

    // Wrap the provider in an Arc right away so we can clone it
    let provider_arc: Arc<dyn ModelProvider> = Arc::from(provider);

    let app_state = AppState {
        provider: provider_arc.clone(),
        log_config: Arc::new(log_config),
    };

    // Handle CLI command if present, otherwise start gRPC server
    match cli.prompt {
        Some(_) => {
            handle_cli(cli, provider_arc).await?;
        }
        None => {
            info!("No prompt provided, starting gRPC server...");

            // Start the gRPC server
            let grpc_server = tokio::spawn(async move {
                if let Err(e) = start_sentence_grpc_server(provider_arc.clone()).await {
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
