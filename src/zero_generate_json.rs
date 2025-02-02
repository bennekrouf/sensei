use crate::{call_ollama::call_ollama, prompts::PromptManager};
use regex::Regex;
use std::error::Error;
use tracing::{debug, error, info};

pub async fn generate_json(
    model: &str,
    sentence: &str,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let prompt_manager = PromptManager::new().await?;
    let full_prompt = prompt_manager.format_generate_json(sentence);

    let full_response_text = call_ollama(&model, &full_prompt).await?;
    debug!("Raw LLM response:\n{}", full_response_text);

    // Extract JSON using regex
    let re = Regex::new(r"\{[\s\S]*\}")?;
    let json_str = re
        .find(&full_response_text)
        .ok_or_else(|| {
            error!("No JSON found in response: {}", full_response_text);
            "No JSON structure found in response"
        })?
        .as_str();

    debug!("Extracted JSON string:\n{}", json_str);

    // Attempt to parse the JSON
    let parsed_json: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
        error!("Failed to parse JSON: {}\nRaw JSON string: {}", e, json_str);
        format!("Failed to parse JSON: {}. Raw JSON: {}", e, json_str)
    })?;

    // Validate the JSON structure
    if !parsed_json.is_object() || !parsed_json.get("endpoints").is_some() {
        error!("Invalid JSON structure: missing 'endpoints' array");
        return Err("Invalid JSON structure: missing 'endpoints' array".into());
    }

    info!("Successfully generated and validated JSON");
    Ok(parsed_json)
}
