use crate::first_find_closest_endpoint::find_closest_endpoint;
use crate::fourth_match_fields::match_fields_semantic;
use crate::models::config::load_models_config;
use crate::models::ConfigFile;
use crate::models::Parameter as ServiceParameter;
use crate::zero_sentence_to_json::sentence_to_json;
use serde_json::Value;
use std::error::Error;
use tracing::{debug, info};

pub struct AnalysisResult {
    pub json_output: Value,
    pub endpoint_id: String,
    pub endpoint_description: String,
    pub parameters: Vec<ServiceParameter>,
}

pub async fn analyze_sentence(
    sentence: &str,
) -> Result<AnalysisResult, Box<dyn Error + Send + Sync>> {
    info!("Starting sentence analysis for: {}", sentence);

    // Load configurations
    let config_str = tokio::fs::read_to_string("endpoints.yaml").await?;
    debug!("Config file content length: {}", config_str.len());

    let config: ConfigFile = serde_yaml::from_str(&config_str)?;
    info!(
        "Configuration loaded with {} endpoints",
        config.endpoints.len()
    );

    // Load model configurations
    let models_config = load_models_config().await?;
    debug!("Model configurations loaded successfully");

    // Generate JSON from sentence
    info!("Generating JSON from sentence");
    let json_result = sentence_to_json(sentence).await?;
    debug!("JSON generation successful");

    // Find closest matching endpoint
    info!("Finding closest matching endpoint");
    let endpoint_result = find_closest_endpoint(&config, sentence).await?;
    debug!("Endpoint matching successful: {}", endpoint_result.id);

    // Perform both exact and semantic matching
    let mut parameters = Vec::new();

    // Exact matching
    for param in &endpoint_result.parameters {
        let mut exact_value = None;
        if let Some(endpoints) = json_result.get("endpoints") {
            if let Some(first_endpoint) = endpoints.as_array().and_then(|arr| arr.first()) {
                if let Some(fields) = first_endpoint.get("fields") {
                    exact_value = fields.get(&param.name).map(|v| v.to_string());
                }
            }
        }

        // Semantic matching
        let semantic_results = match_fields_semantic(&json_result, &endpoint_result).await?;
        let semantic_value = semantic_results
            .iter()
            .find(|(name, _, _)| name == &param.name)
            .and_then(|(_, _, value)| value.clone());

        parameters.push(ServiceParameter {
            name: param.name.clone(),
            description: param.description.clone(),
            semantic_value,
        });
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
            }
            Err(e) => panic!("Analysis failed: {}", e),
        }
    }
}
