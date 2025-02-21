use crate::models::config::load_models_config;
use crate::models::Endpoint;
use crate::{call_ollama::call_ollama_with_config, json_helper::sanitize_json};
use serde_json::Value;
use std::error::Error;
use tracing::{debug, info};

pub async fn match_fields_semantic(
    input_json: &Value,
    endpoint: &Endpoint,
) -> Result<Vec<(String, String, Option<String>)>, Box<dyn Error + Send + Sync>> {
    let input_fields = if let Some(endpoints) = input_json.get("endpoints") {
        if let Some(first_endpoint) = endpoints.as_array().and_then(|arr| arr.first()) {
            if let Some(fields) = first_endpoint.get("fields") {
                if let Some(obj) = fields.as_object() {
                    obj.iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect::<Vec<_>>()
                        .join(", ")
                } else {
                    return Err("Fields is not an object".into());
                }
            } else {
                return Err("No fields found in JSON".into());
            }
        } else {
            return Err("No endpoints found in JSON".into());
        }
    } else {
        return Err("Invalid JSON structure".into());
    };

    let parameters = endpoint
        .parameters
        .iter()
        .map(|p| {
            format!(
                "{}: {} (alternatives: {})",
                p.name,
                p.description,
                p.alternatives.join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "Given these input fields from a sentence: '{input_fields}'\n\
        And these endpoint parameters:\n{parameters}\n\n\
        For each endpoint parameter:\n\
        1. Look at the input fields\n\
        2. Find any field that matches the parameter or its alternatives\n\
        3. Extract the actual value from the matching input field\n\n\
        Return a JSON where:\n\
        - Keys are the endpoint parameter names\n\
        - Values are the actual values found in the input fields\n\
        Only include parameters where you found a matching value.\n\
        Return valid JSON only, no additional text.",
    );

    debug!("Field matching prompt:\n{}", prompt);
    info!("Calling Ollama for field matching");

    // Load model configuration
    let models_config = load_models_config().await?;
    let model_config = &models_config.sentence_to_json;

    let response = call_ollama_with_config(model_config, &prompt).await?;
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
            for alt in &param.alternatives {
                if let Some(v) = input_fields.get(alt) {
                    value = Some(v.to_string());
                    break;
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
