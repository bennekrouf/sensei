// src/cli.rs - Updated to make email required
use clap::{Parser, ValueEnum};
use std::{error::Error, sync::Arc};
use tracing::{error, info};

use crate::endpoint_client::get_default_api_url;
use crate::utils::email::validate_email;
use crate::{analyze_sentence::analyze_sentence, models::providers::ModelProvider};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
#[clap(rename_all = "lowercase")]
pub enum ProviderType {
    /// Use Ollama (local models)
    Ollama,
    /// Use Claude API (requires API key in .env)
    Claude,
}

pub fn display_custom_help() {
    println!("
╭───────────────────────────────────────────────╮
│                  Semantic                      │
│         Natural Language API Matcher           │
╰───────────────────────────────────────────────╯

ARGUMENTS:
  --provider TYPE    Select which LLM provider to use (default: claude)
                     Options: ollama, claude

  --email ADDRESS    Your email address 
                     (REQUIRED ONLY when analyzing a sentence)

  --api URL          Remote API endpoint for fetching endpoints
                     Default: Uses local endpoints.yaml
  
  --port PORT        Override gRPC server port
                     Default: From config.yaml

USAGE EXAMPLES:
  1. Start gRPC server (no email required):
     semantic --provider ollama
  
  2. Analyze text (email required):
     semantic --email user@example.com \"analyze this text\"
  
  3. Use remote endpoints:
     semantic --provider claude --api http://example.com:50053 --email user@example.com \"analyze this\"

For more information, use the standard help:
  semantic --help
");
}

// Update the Cli struct with a help_template override
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(help_template = "\
{before-help}{name} {version}
{author}
{about}

[REQUIRED PARAMETERS]
--provider TYPE    : ollama or claude
--email ADDRESS    : valid email address

{usage-heading} {usage}

{all-args}{after-help}
")]
pub struct Cli {
    /// The sentence to analyze (if not provided, starts gRPC server)
    pub prompt: Option<String>,

    /// Select which LLM provider to use: 'ollama' or 'claude'
    #[arg(long, value_enum, value_name = "TYPE", default_value = "claude")]
    pub provider: ProviderType,

    /// Remote API endpoint for fetching endpoint definitions (optional)
    #[arg(long, value_name = "URL")]
    pub api: Option<String>,

    /// Email address for authentication (required when analyzing a sentence)
    #[arg(
        long,
        value_name = "EMAIL",
        help = "Email address (required when analyzing a sentence)"
    )]
    pub email: Option<String>,

    /// Override gRPC server port (default from config.yaml)
    #[arg(long, value_name = "PORT")]
    pub port: Option<u16>,
}

pub async fn handle_cli(
    mut cli: Cli,
    provider: Arc<dyn ModelProvider>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(prompt) = cli.prompt.clone() {
        // Email is required when analyzing a sentence
        let email = match &cli.email {
            Some(email) => {
                // Validate email
                if let Err(e) = validate_email(email) {
                    error!("Invalid email: {}", e);
                    return Err(
                        format!("Email is required when analyzing a sentence: {}", e).into(),
                    );
                }
                email.clone()
            }
            None => {
                error!("Email is required when analyzing a sentence");
                return Err(
                    "Email is required when analyzing a sentence. Please provide it with --email"
                        .into(),
                );
            }
        };

        match cli.provider {
            ProviderType::Claude => {
                info!("Using Claude API for analysis");
            }
            ProviderType::Ollama => {
                info!("Using self-hosted Ollama models for analysis");
            }
        };

        // If API URL not provided in CLI, try to get default from config
        if cli.api.is_none() {
            match get_default_api_url().await {
                Ok(url) => {
                    info!("Using default API URL from config: {}", url);
                    cli.api = Some(url);
                }
                Err(e) => {
                    info!(
                        "Could not get default API URL, using local endpoints: {}",
                        e
                    );
                }
            }
        }

        let endpoint_source = match &cli.api {
            Some(api_url) => format!("remote API ({})", api_url),
            None => "local file".to_string(),
        };

        info!("Using endpoints from {}", endpoint_source);
        info!("Analyzing prompt via CLI: {}", prompt);

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
