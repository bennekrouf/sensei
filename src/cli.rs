// src/cli.rs
use crate::analyze_sentence::analyze_sentence;
use clap::Parser;
use std::error::Error;
use tracing::info;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The sentence to analyze (if not provided, starts gRPC server)
    pub prompt: Option<String>,
}

pub async fn handle_cli(cli: Cli) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(prompt) = cli.prompt {
        info!("Analyzing prompt via CLI: {}", prompt);
        let result = analyze_sentence(&prompt).await?;

        println!("\nAnalysis Results:");
        println!(
            "Endpoint: {} ({})",
            result.endpoint_id, result.endpoint_description
        );
        println!("\nParameters:");
        for param in result.parameters {
            println!("\n{} ({}):", param.name, param.description);
            if let Some(semantic) = param.semantic_value {
                println!("  Semantic Match: {}", semantic);
            }
        }

        println!("\nRaw JSON Output:");
        println!("{}", serde_json::to_string_pretty(&result.json_output)?);
    }
    Ok(())
}
