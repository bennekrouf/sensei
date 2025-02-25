mod analyze_sentence;
mod call_ollama;
mod cli;
mod grpc_server;
mod json_helper;
mod models;
mod prompts;
mod sentence_service;
//use workflow::{find_closest_endpoint, match_fields_semantic, sentence_to_json};
mod workflow;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};
use clap::Parser;
use cli::{handle_cli, Cli};
use grpc_server::start_sentence_grpc_server;
use std::error::Error;
use tokio::signal;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // let config = load_config("config.yaml")?;
    // setup_logging(&config).await?;

    Registry::default()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("INFO")))
        .init();
    // Parse CLI arguments
    let cli = Cli::parse();

    // Handle CLI command if present, otherwise start gRPC server
    match cli.prompt {
        Some(_) => {
            handle_cli(cli).await?;
        }
        None => {
            info!("No prompt provided, starting gRPC server...");

            // Start the gRPC server
            let grpc_server = tokio::spawn(async {
                if let Err(e) = start_sentence_grpc_server().await {
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
