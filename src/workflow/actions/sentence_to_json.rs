use crate::json_helper::sanitize_json;
use crate::models::config::load_models_config;
use crate::{call_ollama::call_ollama_with_config, prompts::PromptManager};
use std::error::Error;
use tracing::{debug, error, info};

pub async fn sentence_to_json(
    sentence: &str,
) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
    let prompt_manager = PromptManager::new().await?;
    let full_prompt = prompt_manager.format_sentence_to_json(sentence, Some("v2"));

    // Load model configuration
    let models_config = load_models_config().await?;
    let model_config = &models_config.sentence_to_json;

    let full_response_text = call_ollama_with_config(model_config, &full_prompt).await?;
    debug!("Raw LLM response:\n{}", full_response_text);

    let parsed_json = sanitize_json(&full_response_text)?;

    // Validate the JSON structure has endpoints array
    if !parsed_json.is_object() || !parsed_json.get("endpoints").is_some() {
        error!("Invalid JSON structure: missing 'endpoints' array");
        return Err("Invalid JSON structure: missing 'endpoints' array".into());
    }

    // Ensure endpoints is an array
    let endpoints = parsed_json
        .get("endpoints")
        .and_then(|e| e.as_array())
        .ok_or_else(|| {
            error!("Invalid JSON structure: 'endpoints' is not an array");
            "Invalid JSON structure: 'endpoints' is not an array"
        })?;

    // Validate each endpoint in the array
    for (i, endpoint) in endpoints.iter().enumerate() {
        if !endpoint.is_object() {
            error!("Invalid endpoint structure at index {}", i);
            return Err(format!("Invalid endpoint structure at index {}", i).into());
        }

        let fields = endpoint.get("fields").ok_or_else(|| {
            error!("Missing fields object in endpoint at index {}", i);
            format!("Missing fields object in endpoint at index {}", i)
        })?;

        if !fields.is_object() {
            error!("Invalid fields structure at index {}", i);
            return Err(format!("Invalid fields structure at index {}", i).into());
        }
    }

    info!(
        "Successfully generated and validated JSON with {} endpoints",
        endpoints.len()
    );
    Ok(parsed_json)
}
