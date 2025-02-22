use crate::first_find_closest_endpoint::find_closest_endpoint;
use crate::fourth_match_fields::match_fields_semantic;
use crate::models::config::load_models_config;
use crate::models::config::ModelsConfig;
use crate::models::ConfigFile;
use crate::models::Endpoint;
use crate::models::Parameter as ServiceParameter;
use crate::workflow::WorkflowConfig;
use crate::workflow::WorkflowEngine;
use crate::workflow::WorkflowStep;
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

use async_trait::async_trait;
//use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::error;

// Step 1: Enhanced WorkflowContext to hold all necessary state
#[derive(Clone, Debug, Default)]
pub struct WorkflowContext {
    // Input
    pub sentence: String,

    // Configurations
    pub models_config: Option<ModelsConfig>,
    pub endpoints_config: Option<ConfigFile>,

    // Processing state
    pub json_output: Option<serde_json::Value>,
    pub matched_endpoint: Option<Endpoint>,
    pub parameters: Vec<ServiceParameter>,
    pub endpoint_id: Option<String>,
    pub endpoint_description: Option<String>,
}

// Step 2: Define each workflow step

// Step 2.1: Configuration Loading Step
pub struct ConfigurationLoadingStep;

#[async_trait]
impl WorkflowStep for ConfigurationLoadingStep {
    async fn execute(
        &self,
        context: &mut WorkflowContext,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Loading configurations");

        // Load endpoints configuration
        let config_str = tokio::fs::read_to_string("endpoints.yaml").await?;
        let config: ConfigFile = serde_yaml::from_str(&config_str)?;
        context.endpoints_config = Some(config);

        // Load model configurations
        let models_config = load_models_config().await?;
        context.models_config = Some(models_config);

        debug!("Configurations loaded successfully");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "configuration_loading"
    }
}

// Step 2.2: JSON Generation Step
pub struct JsonGenerationStep;

#[async_trait]
impl WorkflowStep for JsonGenerationStep {
    async fn execute(
        &self,
        context: &mut WorkflowContext,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Generating JSON from sentence");

        let json_result = sentence_to_json(&context.sentence).await?;
        context.json_output = Some(json_result);

        debug!("JSON generation successful");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "json_generation"
    }
}

// Step 2.3: Endpoint Matching Step
pub struct EndpointMatchingStep;

#[async_trait]
impl WorkflowStep for EndpointMatchingStep {
    async fn execute(
        &self,
        context: &mut WorkflowContext,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Finding closest matching endpoint");

        let config = context
            .endpoints_config
            .as_ref()
            .ok_or("Endpoints configuration not loaded")?;

        let endpoint_result = find_closest_endpoint(config, &context.sentence).await?;

        context.endpoint_id = Some(endpoint_result.id.clone());
        context.endpoint_description = Some(endpoint_result.description.clone());
        context.matched_endpoint = Some(endpoint_result);

        debug!("Endpoint matching successful");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "endpoint_matching"
    }
}

// Step 2.4: Field Matching Step
pub struct FieldMatchingStep;

#[async_trait]
impl WorkflowStep for FieldMatchingStep {
    async fn execute(
        &self,
        context: &mut WorkflowContext,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Performing field matching");

        let json_output = context
            .json_output
            .as_ref()
            .ok_or("JSON output not available")?;
        let endpoint = context
            .matched_endpoint
            .as_ref()
            .ok_or("Matched endpoint not available")?;

        let semantic_results = match_fields_semantic(json_output, endpoint).await?;

        // Convert semantic results to parameters
        let parameters: Vec<ServiceParameter> = endpoint
            .parameters
            .iter()
            .map(|param| {
                let semantic_value = semantic_results
                    .iter()
                    .find(|(name, _, _)| name == &param.name)
                    .and_then(|(_, _, value)| value.clone());

                ServiceParameter {
                    name: param.name.clone(),
                    description: param.description.clone(),
                    semantic_value,
                }
            })
            .collect();

        context.parameters = parameters;

        debug!("Field matching completed");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "field_matching"
    }
}

// Step 3: Workflow Configuration
const WORKFLOW_CONFIG: &str = r#"
steps:
  - name: configuration_loading
    enabled: true
    retry:
      max_attempts: 3
      delay_ms: 1000
    timeout_secs: 10
  - name: json_generation
    enabled: true
    retry:
      max_attempts: 3
      delay_ms: 1000
    timeout_secs: 30
  - name: endpoint_matching
    enabled: true
    retry:
      max_attempts: 2
      delay_ms: 500
    timeout_secs: 20
  - name: field_matching
    enabled: true
    retry:
      max_attempts: 2
      delay_ms: 500
    timeout_secs: 20
"#;

// Step 4: Updated analyze_sentence function
pub async fn analyze_sentence(
    sentence: &str,
) -> Result<AnalysisResult, Box<dyn Error + Send + Sync>> {
    info!("Starting sentence analysis for: {}", sentence);

    // Initialize workflow engine
    let config: WorkflowConfig = serde_yaml::from_str(WORKFLOW_CONFIG)?;
    let mut engine = WorkflowEngine::new();

    // Register all steps
    for step_config in config.steps {
        match step_config.name.as_str() {
            "configuration_loading" => {
                engine.register_step(step_config, Arc::new(ConfigurationLoadingStep));
            }
            "json_generation" => {
                engine.register_step(step_config, Arc::new(JsonGenerationStep));
            }
            "endpoint_matching" => {
                engine.register_step(step_config, Arc::new(EndpointMatchingStep));
            }
            "field_matching" => {
                engine.register_step(step_config, Arc::new(FieldMatchingStep));
            }
            _ => {
                error!("Unknown step: {}", step_config.name);
                return Err(format!("Unknown step: {}", step_config.name).into());
            }
        }
    }

    // Execute workflow
    let context = engine.execute(sentence.to_string()).await?;

    // Convert workflow context to analysis result
    Ok(AnalysisResult {
        json_output: context.json_output.ok_or("JSON output not available")?,
        endpoint_id: context.endpoint_id.ok_or("Endpoint ID not available")?,
        endpoint_description: context
            .endpoint_description
            .ok_or("Endpoint description not available")?,
        parameters: context.parameters,
    })
}

// Step 5: Tests to verify functionality
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_workflow_execution() {
        let test_sentence = "schedule a meeting tomorrow at 2pm with John";
        match analyze_sentence(test_sentence).await {
            Ok(result) => {
                assert!(!result.endpoint_id.is_empty());
                assert!(!result.endpoint_description.is_empty());
                assert!(!result.parameters.is_empty());

                // Verify correct endpoint was matched
                assert_eq!(result.endpoint_id, "schedule_meeting");

                // Verify parameters were extracted
                let has_time = result.parameters.iter().any(|p| p.name == "time");
                let has_participants = result.parameters.iter().any(|p| p.name == "participants");

                assert!(has_time);
                assert!(has_participants);
            }
            Err(e) => panic!("Workflow execution failed: {}", e),
        }
    }
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
