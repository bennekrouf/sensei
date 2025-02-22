use crate::models::config::ModelConfig;
use crate::models::GenerateRequest;
use crate::models::OllamaResponse;

use std::error::Error;
use tracing::{debug, error, info};

pub async fn call_ollama_with_config(
    model_config: &ModelConfig,
    prompt: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    debug!("Creating Ollama request for model: {}", model_config.name);
    let client = reqwest::Client::new();
    let request_body = GenerateRequest {
        model: model_config.name.clone(),
        prompt: prompt.to_string(),
        stream: false,
        format: None,
        temperature: model_config.temperature,
        max_tokens: model_config.max_tokens,
    };

    info!("Sending request to Ollama : {:?}", &request_body);
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request_body)
        .send()
        .await?;

    debug!("Response status: {}", response.status());

    if !response.status().is_success() {
        error!("Ollama request failed with status: {}", response.status());
        return Err(format!("Request failed with status: {}", response.status()).into());
    }

    let response_obj = response.json::<OllamaResponse>().await?;

    if response_obj.response.trim().is_empty() {
        error!("Received empty response from Ollama");
        return Err("Empty response from Ollama".into());
    }

    debug!("Parsed response: '{}'", response_obj.response.trim());

    Ok(response_obj.response.trim().to_owned())
}
