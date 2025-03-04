// src/models/providers/mod.rs - Update ModelConfig struct

use async_trait::async_trait;
use serde::Deserialize;
use std::error::Error;

pub mod claude;
pub mod ollama;
mod selector;

pub use selector::ProviderSelector;

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn generate(
        &self,
        prompt: &str,
        model: &ModelConfig,
    ) -> Result<String, Box<dyn Error + Send + Sync>>;
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ProviderConfig {
    pub enabled: bool,
    pub host: Option<String>,
    pub api_key: Option<String>,
    pub models: ModelsConfig, 
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProvidersConfig {
    pub ollama: ProviderConfig,
    pub claude: ProviderConfig,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ModelConfig {
    #[serde(default)]
    pub name: String, // Keep for backward compatibility
    #[serde(default)]
    pub ollama: String,
    #[serde(default)]
    pub claude: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ModelsConfig {
    pub sentence_to_json: ModelConfig,
    pub find_endpoint: ModelConfig,
    pub semantic_match: ModelConfig,
}

use dotenv::dotenv;

// Initialize environment
pub fn init_environment() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenv().ok();
    Ok(())
}

// Factory function to create provider instance
pub fn create_provider(config: &ProviderConfig) -> Option<Box<dyn ModelProvider>> {
    match config.enabled {
        true => {
            if config.api_key.is_some() {
                Some(Box::new(claude::ClaudeProvider::new(config)))
            } else if config.host.is_some() {
                Some(Box::new(ollama::OllamaProvider::new(config)))
            } else {
                None
            }
        }
        false => None,
    }
}
