// src/models/providers/claude.rs

use super::{ModelConfig, ModelProvider, ProviderConfig, ProviderSelector};
use async_trait::async_trait;
use serde::Serialize;
use std::error::Error;
use tracing::{debug, info};

pub struct ClaudeProvider {
    api_key: String,
}

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

impl ClaudeProvider {
    pub fn new(config: &ProviderConfig) -> Self {
        Self {
            api_key: config
                .api_key
                .clone()
                .expect("Claude API key not specified"),
        }
    }
}

#[async_trait]
impl ModelProvider for ClaudeProvider {
    async fn generate(
        &self,
        prompt: &str,
        config: &ModelConfig,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        debug!("Generating response with Claude API");
        
        // Get the appropriate Claude model name
        let model_name = ProviderSelector::get_model_name(config, true);
        
        let request = ClaudeRequest {
            model: model_name,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: config.temperature,
            max_tokens: config.max_tokens,
        };

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;

        println!(
            "Claude API response: {}",
            serde_json::to_string_pretty(&response_json).unwrap()
        );

        let content = response_json["content"][0]["text"]
            .as_str()
            .ok_or("Invalid response format")?
            .to_string();

        info!("Successfully received response from Claude API");
        Ok(content)
    }
}
