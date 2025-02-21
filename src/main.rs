mod analyze_sentence;
mod call_ollama;
mod cli;
//mod example_usage_with_json;
mod first_find_closest_endpoint;
mod fourth_match_fields;
mod grpc_server;
mod json_helper;
mod models;
mod prompts;
mod second_extract_matched_action;
mod sentence_service;
mod third_find_endpoint_by_substring;
mod zero_sentence_to_json;

// use example_usage_with_json::example_usage_with_json;
use clap::Parser;
use cli::{handle_cli, Cli};
use grpc_logger::load_config;
use grpc_logger::setup_logging;
use grpc_server::start_sentence_grpc_server;
use std::error::Error;
use tokio::signal;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let config = load_config("config.yaml")?;
    setup_logging(&config).await?;

    // Parse CLI arguments
    let cli = Cli::parse();

    // Start the gRPC server
    //let grpc_server = tokio::spawn(async {
    //    if let Err(e) = start_sentence_grpc_server().await {
    //        error!("gRPC server error: {:?}", e);
    //    }
    //});

    //info!("Semantic server started");

    // Example task (if needed)
    // let example_task = tokio::spawn(async {
    //     if let Err(e) = example_usage_with_json().await {
    //         info!("Error in semantic application: {:?}", e);
    //     }
    // });

    // Wait for either CTRL-C or task completion
    //tokio::select! {
    //    _ = signal::ctrl_c() => {
    //        info!("Received shutdown signal, initiating graceful shutdown...");
    //    }
    //    result = grpc_server => {
    //        if let Err(e) = result {
    //            error!("gRPC server task error: {:?}", e);
    //        }
    //    }
    // result = example_task => {
    //     if let Err(e) = result {
    //         info!("Example task error: {:?}", e);
    //     }
    //    // }
    //}

    //info!("Server shutting down");

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
