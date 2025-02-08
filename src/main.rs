mod call_ollama;
mod example_usage_with_json;
mod first_find_closest_endpoint;
mod models;
mod prompts;
mod second_extract_matched_action;
mod third_find_endpoint_by_substring;
mod zero_sentence_to_json;

use example_usage_with_json::example_usage_with_json;
use grpc_logger::LoggingService;
use std::error::Error;
use tracing::info;

use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging configuration
    let config = grpc_logger::load_config("config.yaml")?;
    let service = LoggingService::new();
    service.init(&config).await?;

    // Spawn the semantic application task
    let semantic_task = tokio::spawn(async move {
        if let Err(e) = example_usage_with_json().await {
            info!("Error in semantic application: {:?}", e);
        }
    });

    // Wait for either CTRL-C or task completion
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received shutdown signal, initiating graceful shutdown...");
        }
        result = semantic_task => {
            if let Err(e) = result {
                info!("Semantic task error: {:?}", e);
            }
        }
    }

    info!("Server shutting down");
    Ok(())
}
