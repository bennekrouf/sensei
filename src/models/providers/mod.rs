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
    //pub models: ModelsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProvidersConfig {}

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
}

pub fn create_provider(config: &ProviderConfig) -> Option<Box<dyn ModelProvider>> {
    // We check the enabled flag to avoid warnings about it not being used
    if !config.enabled {
        return None;
    }

    // Use the provider determination logic
    if config.api_key.is_some() {
        Some(Box::new(claude::ClaudeProvider::new(config)))
    } else if config.host.is_some() {
        Some(Box::new(ollama::OllamaProvider::new(config)))
    } else {
        None
    }
}
