use crate::json_helper::sanitize_json;
use crate::models::config::load_models_config;
use crate::models::providers::ModelProvider;

use crate::prompts::PromptManager;
use std::{error::Error, sync::Arc};
use tracing::{debug, error, info};

pub async fn sentence_to_json(
    sentence: &str,
    provider: Arc<dyn ModelProvider>,
) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
    let prompt_manager = PromptManager::new().await?;
    let full_prompt = prompt_manager.format_sentence_to_json(sentence, Some("v1"));

    // Load model configuration
    let models_config = load_models_config().await?;
    let model_config = &models_config.sentence_to_json;

    // Use correct type
    let full_response_text = provider.generate(&full_prompt, model_config).await?;
    debug!("Raw LLM response:\n{}", full_response_text);

    let parsed_json = sanitize_json(&full_response_text)?;

    // Validate the JSON structure
    if !parsed_json.is_object() || !parsed_json.get("endpoints").is_some() {
        error!("Invalid JSON structure: missing 'endpoints' array");
        return Err("Invalid JSON structure: missing 'endpoints' array".into());
    }

    // Additional validation to ensure endpoints is an array and has at least one item
    let endpoints = parsed_json
        .get("endpoints")
        .and_then(|e| e.as_array())
        .ok_or_else(|| {
            error!("Invalid JSON structure: 'endpoints' is not an array");
            "Invalid JSON structure: 'endpoints' is not an array"
        })?;

    if endpoints.is_empty() {
        error!("Invalid JSON structure: 'endpoints' array is empty");
        return Err("Invalid JSON structure: 'endpoints' array is empty".into());
    }

    info!("Successfully generated and validated JSON");
    Ok(parsed_json)
}
