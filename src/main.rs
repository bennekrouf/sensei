mod call_ollama;
mod example_usage_with_json;
mod first_find_closest_endpoint;
mod models;
mod prompts;
mod second_extract_matched_action;
mod third_find_endpoint_by_substring;
mod zero_sentence_to_json;
mod analyze_sentence;
mod sentence_service;
mod grpc_server;

// use example_usage_with_json::example_usage_with_json;
use grpc_server::start_sentence_grpc_server;
use std::error::Error;
use tracing::{error, info};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
// Set up client logging
    // grpc_logger::setup_client_logging(&config)?;
    // Initialize logging configuration
    let config = grpc_logger::load_config("config.yaml")?;
    // let service = LoggingService::new();
    // service.init(&config).await?;

    // Set up the logging subscriber without creating a new server
    // grpc_logger::setup_logging(&config, None)?;

    // Set up client logging
    grpc_logger::setup_client_logging(&config).await?;

    // Start the gRPC server
    let grpc_server = tokio::spawn(async {
        if let Err(e) = start_sentence_grpc_server().await {
            error!("gRPC server error: {:?}", e);
        }
    });

    // Example task (if needed)
    // let example_task = tokio::spawn(async {
    //     if let Err(e) = example_usage_with_json().await {
    //         info!("Error in semantic application: {:?}", e);
    //     }
    // });

    // Wait for either CTRL-C or task completion
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received shutdown signal, initiating graceful shutdown...");
        }
        result = grpc_server => {
            if let Err(e) = result {
                error!("gRPC server task error: {:?}", e);
            }
        }
        // result = example_task => {
        //     if let Err(e) = result {
        //         info!("Example task error: {:?}", e);
        //     }
        // }
    }

    info!("Server shutting down");
    Ok(())
}
