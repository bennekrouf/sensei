use crate::models::config::load_models_config;
use crate::models::Endpoint;
use crate::prompts::PromptManager;
use crate::{call_ollama::call_ollama_with_config, json_helper::sanitize_json};
use serde_json::Value;
use std::error::Error;
use tracing::debug;

use std::sync::Arc;
 use crate::ModelProvider;
pub async fn match_fields_semantic(
    input_json: &Value,
    endpoint: &Endpoint,
    provider: Arc<dyn ModelProvider>,  // Add this parameter
) -> Result<Vec<(String, String, Option<String>)>, Box<dyn Error + Send + Sync>> {
    let input_fields = input_json
        .get("endpoints")
        .ok_or("Invalid JSON structure")?
        .as_array()
        .and_then(|arr| arr.first())
        .ok_or("No endpoints found in JSON")?
        .get("fields")
        .ok_or("No fields found in JSON")?
        .as_object()
        .ok_or("Fields is not an object")?
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<_>>()
        .join(", ");

    let parameters = endpoint
        .parameters
        .iter()
        .map(|p| format!("{}: {}", p.name, p.description,))
        .collect::<Vec<_>>()
        .join("\n");

    // Initialize PromptManager and get the match_fields template
    let prompt_manager = PromptManager::new().await?;
    let template = prompt_manager
        .get_prompt("match_fields", Some("v1"))
        .ok_or("Match fields prompt template not found")?;

    // Replace placeholders in the template
    let prompt = template
        .replace("{input_fields}", &input_fields)
        .replace("{parameters}", &parameters);

    debug!("Field matching prompt:\n{}", prompt);
    debug!("Calling Ollama for field matching");
    // Load model configuration
    let models_config = load_models_config().await?;
    let model_config = &models_config.sentence_to_json;

    // let response = call_ollama_with_config(model_config, &prompt).await?;

    let response = provider.generate(&prompt, model_config).await?;

    let json_response = sanitize_json(&response)?;

    debug!("Semantic matching response: {:?}", json_response);

    let mut matched_fields = Vec::new();
    let input_fields = input_json["endpoints"][0]["fields"]
        .as_object()
        .ok_or("Invalid JSON structure")?;

    for param in &endpoint.parameters {
        // First try exact match
        let mut value = input_fields.get(&param.name).map(|v| v.to_string());

        // If no exact match, try alternatives
        if value.is_none() {
            if let Some(alternatives) = &param.alternatives {
                for alt in alternatives {
                    // Changed this line
                    if let Some(v) = input_fields.get(alt) {
                        // Now alt is a &String
                        value = Some(v.to_string());
                        break;
                    }
                }
            }
        }

        // If still no match, check semantic matching result
        if value.is_none() {
            if let Some(v) = json_response.get(&param.name) {
                value = Some(v.to_string().trim_matches('"').to_string());
            }
        }

        matched_fields.push((param.name.clone(), param.description.clone(), value));
    }

    Ok(matched_fields)
}
