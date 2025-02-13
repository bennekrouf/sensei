use crate::models::ConfigFile;
use crate::zero_sentence_to_json::sentence_to_json;
use crate::first_find_closest_endpoint::find_closest_endpoint;
use std::error::Error;
use tracing::{debug, info};
use serde_json::Value;

pub struct AnalysisResult {
    pub json_output: Value,
    pub endpoint_id: String,
    pub endpoint_description: String,
    pub parameters: Vec<(String, String, Option<String>)>, // (name, description, value)
}

pub async fn analyze_sentence(sentence: &str) -> Result<AnalysisResult, Box<dyn Error + Send + Sync>> {
    info!("Starting sentence analysis for: {}", sentence);

    // Load configuration
    let config_str = tokio::fs::read_to_string("endpoints.yaml").await?;
    debug!("Config file content length: {}", config_str.len());

    let config: ConfigFile = serde_yaml::from_str(&config_str)?;
    info!(
        "Configuration loaded with {} endpoints",
        config.endpoints.len()
    );

    // Generate JSON from sentence
    info!("Generating JSON from sentence");
    let json_result = sentence_to_json("llama2", sentence).await?;
    debug!("JSON generation successful");

    // Find closest matching endpoint
    info!("Finding closest matching endpoint");
    let endpoint_result = find_closest_endpoint(&config, sentence, "deepseek-r1:8b").await?;
    debug!("Endpoint matching successful: {}", endpoint_result.id);

    // Extract parameter values from JSON
    let mut parameters = Vec::new();
    for param in &endpoint_result.parameters {
        let mut value = None;
        if let Some(endpoints) = json_result.get("endpoints") {
            if let Some(first_endpoint) = endpoints.as_array().and_then(|arr| arr.first()) {
                if let Some(fields) = first_endpoint.get("fields") {
                    value = fields.get(&param.name).map(|v| v.to_string());
                }
            }
        }
        parameters.push((
            param.name.clone(),
            param.description.clone(),
            value,
        ));
    }

    debug!("Analysis completed successfully");
    Ok(AnalysisResult {
        json_output: json_result,
        endpoint_id: endpoint_result.id,
        endpoint_description: endpoint_result.description,
        parameters,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyze_sentence() {
        let test_sentence = "schedule a meeting tomorrow at 2pm with John";
        match analyze_sentence(test_sentence).await {
            Ok(result) => {
                assert!(!result.endpoint_id.is_empty());
                assert!(!result.endpoint_description.is_empty());
                // Add more assertions as needed
            },
            Err(e) => panic!("Analysis failed: {}", e),
        }
    }
}
