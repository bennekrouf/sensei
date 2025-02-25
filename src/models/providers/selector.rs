// src/models/providers/selector.rs

use super::{ModelsConfig, ProviderConfig};

use super::claude::ClaudeProvider;
use super::ollama::OllamaProvider;
use crate::ModelProvider;
use dotenv::dotenv;
use std::env;
use tracing::{error, info};
pub struct ProviderSelector;

impl ProviderSelector {
    pub fn select_provider(use_claude: bool) -> Box<dyn ModelProvider> {
        if use_claude {
            // Load .env file and check for Claude API key
            dotenv().ok();
            match env::var("CLAUDE_API_KEY") {
                Ok(api_key) => {
                    info!("Using Claude API");
                    let config = ProviderConfig {
                        enabled: true,
                        api_key: Some(api_key),
                        host: None,
                        models: ModelsConfig::default(),
                    };
                    Box::new(ClaudeProvider::new(&config))
                }
                Err(_) => {
                    error!(
                        "Claude API key not found in .env file. Please add CLAUDE_API_KEY to .env"
                    );
                    std::process::exit(1);
                }
            }
        } else {
            // Use self-hosted Ollama
            info!("Using self-hosted Ollama");
            let config = ProviderConfig {
                enabled: true,
                host: Some("http://localhost:11434".to_string()),
                api_key: None,
                models: ModelsConfig::default(), // Add this line
            };
            Box::new(OllamaProvider::new(&config))
        }
    }
}
