mod call_ollama;
mod first_find_closest_endpoint;
mod models;
mod prompts;
mod second_extract_matched_action;
mod third_find_endpoint_by_substring;
mod zero_sentence_to_json;
mod example_usage_with_json;

use std::error::Error;
use example_usage_with_json::example_usage_with_json;
use tracing::info;
use grpc_logger::LoggingService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging configuration
    let config = grpc_logger::load_config("config.yaml")?;
    let service = LoggingService::new();
    service.init(&config).await?;

    // Run the semantic application
    tokio::spawn(async move {
        if let Err(e) = example_usage_with_json().await {
            info!("Error in semantic application: {:?}", e);
        }
    });

    // Keep the main task running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    #[allow(unreachable_code)]
    Ok(())
}

