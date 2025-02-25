// src/cli.rs
use clap::{Parser, ValueEnum};
use std::{error::Error, sync::Arc};
use tracing::info;

use crate::{analyze_sentence::analyze_sentence, models::providers::ModelProvider};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
#[clap(rename_all = "lowercase")]
pub enum ProviderType {
    /// Use Ollama (local models)
    Ollama,
    /// Use Claude API (requires API key in .env)
    Claude,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The sentence to analyze (if not provided, starts gRPC server)
    pub prompt: Option<String>,

    /// Select which LLM provider to use: 'ollama' or 'claude'
    #[arg(long, value_enum, required = true, value_name = "TYPE")]
    pub provider: ProviderType,
}

pub async fn handle_cli(
    cli: Cli,
    provider: Arc<dyn ModelProvider>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(prompt) = cli.prompt {
        match cli.provider {
            ProviderType::Claude => {
                info!("Using Claude API for analysis");
            }
            ProviderType::Ollama => {
                info!("Using self-hosted Ollama models for analysis");
            }
        };

        info!("Analyzing prompt via CLI: {}", prompt);
        let result = analyze_sentence(&prompt, provider).await?;

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
