use crate::call_ollama::call_ollama;
use crate::models::Endpoint;
use serde_json::Value;
use std::error::Error;
use tracing::{debug, info};

pub async fn match_fields(
    input_json: &Value,
    endpoint: &Endpoint,
    model: &str,
) -> Result<Value, Box<dyn Error>> {
    let input_fields = input_json["fields"]
        .as_object()
        .map(|fields| {
            fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();

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
        "Map these input fields to endpoint parameters:\n\
        Input fields: {}\n\
        Available parameters:\n{}\n\
        Return only a JSON with parameter name as key and matched value as value. \
        Use the alternatives list to find matching fields. \
        Return only the parameters that have matching input fields.",
        input_fields, parameters
    );

    debug!("Field matching prompt:\n{}", prompt);
    info!("Calling Ollama for field matching");

    let response = call_ollama(model, &prompt).await?;

    let json_response: Value =
        serde_json::from_str(&response).map_err(|e| Box::new(e) as Box<dyn Error>)?;

    debug!("Field matching response: {:?}", json_response);

    Ok(json_response)
}
