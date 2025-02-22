pub mod config;

use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Debug)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub format: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct OllamaResponse {
    pub response: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Endpoint {
    pub id: String,
    pub text: String,
    pub description: String,
    pub parameters: Vec<EndpointParameter>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EndpointParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub alternatives: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    pub endpoints: Vec<Endpoint>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Parameter {
    pub name: String,
    pub description: String,
    pub semantic_value: Option<String>,
}

impl ConfigFile {
    pub async fn load() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let config_str = tokio::fs::read_to_string("endpoints.yaml").await?;
        let config: ConfigFile = serde_yaml::from_str(&config_str)?;
        Ok(config)
    }
}
