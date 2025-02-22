use crate::{
    models::{config::ModelConfig, ConfigFile},
    prompts::PromptManager,
};

use crate::workflow::context::WorkflowContext;

use super::{
    config::StepConfig, find_closest_endpoint::find_closest_endpoint,
    match_fields::match_fields_semantic, sentence_to_json::sentence_to_json,
};
use std::{error::Error, sync::Arc};

pub struct JsonGenerationStep {
    pub prompt_manager: Arc<PromptManager>,
    pub model_config: ModelConfig,
}

// Trait defining a workflow step
#[async_trait]
pub trait WorkflowStep: Send + Sync {
    async fn execute(
        &self,
        context: &mut WorkflowContext,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
    fn name(&self) -> &'static str;
}

#[async_trait]
impl WorkflowStep for JsonGenerationStep {
    async fn execute(
        &self,
        context: &mut WorkflowContext,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let json_output = sentence_to_json(&context.sentence).await?;
        context.json_output = Some(json_output);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "json_generation"
    }
}

pub struct EndpointMatchingStep {
    pub config: Arc<ConfigFile>,
}

#[async_trait]
impl WorkflowStep for EndpointMatchingStep {
    async fn execute(
        &self,
        context: &mut WorkflowContext,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let endpoint = find_closest_endpoint(&self.config, &context.sentence).await?;
        context.matched_endpoint = Some(endpoint);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "endpoint_matching"
    }
}

use async_trait::async_trait;

use crate::models::EndpointParameter;

// Workflow configuration loaded from YAML

// Workflow engine
pub struct WorkflowEngine {
    steps: Vec<(StepConfig, Arc<dyn WorkflowStep>)>,
}

pub struct FieldMatchingStep {}

#[async_trait]
impl WorkflowStep for FieldMatchingStep {
    async fn execute(
        &self,
        context: &mut WorkflowContext,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let (Some(json), Some(endpoint)) = (&context.json_output, &context.matched_endpoint) {
            let semantic_results = match_fields_semantic(json, endpoint).await?;

            // Convert tuple results into Parameter structs
            let parameters = semantic_results
                .into_iter()
                .map(|(name, description, semantic_value)| EndpointParameter {
                    name,
                    description,
                    semantic_value,
                    alternatives: None,
                    required: None,
                })
                .collect();

            context.parameters = parameters;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "field_matching"
    }
}

// Example workflow configuration in YAML
pub const WORKFLOW_CONFIG: &str = r#"
steps:
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
    timeout_secs: 10
"#;
