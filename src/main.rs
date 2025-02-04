mod call_ollama;
mod first_find_closest_endpoint;
mod models;
mod prompts;
mod second_extract_matched_action;
mod third_find_endpoint_by_substring;
mod zero_sentence_to_json;
mod example_usage_with_json;
use grpc_logger::{setup_logging, LoggingService};
use std::error::Error;
use example_usage_with_json::example_usage_with_json;
use tracing::info;
use grpc_logger::config::LogOutput;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging configuration
    let config = grpc_logger::load_config("config.yaml")?;
    let service = LoggingService::new();

    // Initialize the service
    service.init(&config).await?;

    // let service_clone = service.clone();
    // let _guard = setup_logging(&config, Some(service_clone))?;

    // Log server configuration
    info!(
        "Logger initialized with output: {:?}",
        config.output
    );
    match &config.output {
        LogOutput::File => {
            info!(
                "File logging enabled - path: {}, filename: {}", 
                config.file_path.as_deref().unwrap_or("default"),
                config.file_name.as_deref().unwrap_or("app.log")
            );
        },
        LogOutput::Grpc => {
            if let Some(grpc_config) = &config.grpc {
                info!(
                    "GRPC logging enabled - server running on {}:{}", 
                    grpc_config.address,
                    grpc_config.port
                );
            }
        },
        LogOutput::Console => {
            info!("Console logging enabled");
        }
    }
    info!("Log level set to: {}", config.level);
    info!("Starting semantic application");

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

