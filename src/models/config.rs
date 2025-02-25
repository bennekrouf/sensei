// src/models/config.rs
use crate::models::ModelsConfig;
use serde::Deserialize;
use std::error::Error;
use tracing::debug;

#[derive(Debug, Deserialize, Clone)]
pub struct Provider {
    pub enabled: bool,
    pub host: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Providers {
    pub ollama: Provider,
    pub claude: Provider,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub models: ModelsConfig,
    pub providers: Providers,
}

pub async fn load_models_config() -> Result<ModelsConfig, Box<dyn Error + Send + Sync>> {
    let config_str = tokio::fs::read_to_string("config.yaml").await?;
    let config: Config = serde_yaml::from_str(&config_str)?;

    debug!("Loaded models configuration: {:#?}", config.models);

    Ok(config.models)
}

// Load provider configuration from config file
pub async fn load_providers_config() -> Result<Providers, Box<dyn Error + Send + Sync>> {
    let config_str = tokio::fs::read_to_string("config.yaml").await?;
    let config: Config = serde_yaml::from_str(&config_str)?;

    debug!("Loaded providers configuration: {:#?}", config.providers);

    Ok(config.providers)
}
