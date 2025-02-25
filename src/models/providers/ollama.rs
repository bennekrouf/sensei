use super::{ModelConfig, ModelProvider, ProviderConfig};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::{debug, error, info};

pub struct OllamaProvider {
    host: String,
}

#[derive(Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    format: Option<String>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

impl OllamaProvider {
    pub fn new(config: &ProviderConfig) -> Self {
        Self {
            host: config.host.clone().expect("Ollama host not specified"),
        }
    }
}

#[async_trait]
impl ModelProvider for OllamaProvider {
    async fn generate(
        &self,
        prompt: &str,
        config: &ModelConfig,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let request = GenerateRequest {
            model: config.name.clone(),
            prompt: prompt.to_string(),
            stream: false,
            format: None,
            temperature: config.temperature,
            max_tokens: config.max_tokens,
        };

        debug!("Sending request to Ollama API");
        let response = client
            .post(format!("{}/api/generate", self.host))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_msg = format!("Ollama request failed: {}", response.status());
            error!("{}", error_msg);
            return Err(error_msg.into());
        }

        let response_obj = response.json::<OllamaResponse>().await?;

        if response_obj.response.trim().is_empty() {
            error!("Received empty response from Ollama");
            return Err("Empty response from Ollama".into());
        }

        info!("Successfully received response from Ollama");
        Ok(response_obj.response.trim().to_owned())
    }
}
