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
    
    /// Remote API endpoint for fetching endpoint definitions (optional)
    #[arg(long, value_name = "URL")]
    pub api: Option<String>,
    
    /// Email address for authentication with the remote API
    #[arg(long, value_name = "EMAIL")]
    pub email: Option<String>,
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

        let endpoint_source = match &cli.api {
            Some(api_url) => format!("remote API ({})", api_url),
            None => "local file".to_string(),
        };
        
        info!("Using endpoints from {}", endpoint_source);
        info!("Analyzing prompt via CLI: {}", prompt);
        
        // Get email from CLI or use default
        let email = cli.email.unwrap_or_else(|| "default@example.com".to_string());
        
        // Pass the API URL and email to analyze_sentence
        let result = analyze_sentence(&prompt, provider, cli.api, &email).await?;

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
