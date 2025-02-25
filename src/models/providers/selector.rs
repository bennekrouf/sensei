// src/models/providers/selector.rs

use super::{ModelsConfig, ProviderConfig};

use super::claude::ClaudeProvider;
use super::ollama::OllamaProvider;
use crate::ModelProvider;
use dotenv::dotenv;
use std::env;
use tracing::{error, info, warn};

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
                models: ModelsConfig::default(),
            };
            Box::new(OllamaProvider::new(&config))
        }
    }

    // Get the appropriate model name based on provider type
    pub fn get_model_name(config: &super::ModelConfig, is_claude: bool) -> String {
        if is_claude {
            if !config.claude.is_empty() {
                config.claude.clone()
            } else {
                config.name.clone() // Fallback to generic name
            }
        } else {
            if !config.ollama.is_empty() {
                config.ollama.clone()
            } else {
                config.name.clone() // Fallback to generic name
            }
        }
    }
}
