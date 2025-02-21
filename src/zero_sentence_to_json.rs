use crate::json_helper::sanitize_json;
use crate::models::config::load_models_config;
use crate::{call_ollama::call_ollama_with_config, prompts::PromptManager};
use std::error::Error;
use tracing::{debug, error, info};

pub async fn sentence_to_json(
    sentence: &str,
) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
    let prompt_manager = PromptManager::new().await?;
    let full_prompt = prompt_manager.format_sentence_to_json(sentence);

    // Load model configuration
    let models_config = load_models_config().await?;
    let model_config = &models_config.sentence_to_json;

    let full_response_text = call_ollama_with_config(model_config, &full_prompt).await?;
    debug!("Raw LLM response:\n{}", full_response_text);

    let parsed_json = sanitize_json(&full_response_text)?;

    // Validate the JSON structure - Fixed the condition
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
