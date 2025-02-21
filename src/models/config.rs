use serde::Deserialize;
use std::error::Error;
use tracing::debug;

#[derive(Debug, Deserialize, Clone)]
pub struct ModelConfig {
    pub name: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelsConfig {
    pub sentence_to_json: ModelConfig,
    pub find_endpoint: ModelConfig,
    pub semantic_match: ModelConfig,
}

pub async fn load_models_config() -> Result<ModelsConfig, Box<dyn Error + Send + Sync>> {
    let config_str = tokio::fs::read_to_string("config.yaml").await?;
    let config: serde_yaml::Value = serde_yaml::from_str(&config_str)?;

    let models_config = config["models"]
        .as_mapping()
        .ok_or("No models configuration found")?;

    let models: ModelsConfig = serde_yaml::from_value(config["models"].clone())?;

    debug!("Loaded models configuration: {:#?}", models);

    Ok(models)
}
