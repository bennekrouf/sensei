use async_trait::async_trait;
use serde::Deserialize;
use std::error::Error;
use std::sync::Arc;

use crate::{
    analyze_sentence::WorkflowContext,
    first_find_closest_endpoint::find_closest_endpoint,
    fourth_match_fields::match_fields_semantic,
    models::{
        config::{load_models_config, ModelConfig},
        ConfigFile, EndpointParameter,
    },
    prompts::PromptManager,
    zero_sentence_to_json::sentence_to_json,
};

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

// Example usage
pub async fn analyze_sentence_workflow(
    sentence: &str,
) -> Result<WorkflowContext, Box<dyn Error + Send + Sync>> {
    // Load workflow configuration
    let config: WorkflowConfig = serde_yaml::from_str(WORKFLOW_CONFIG)?;

    // Initialize workflow engine
    let mut engine = WorkflowEngine::new();

    // Register steps with their configurations
    let prompt_manager: Arc<PromptManager> = Arc::new(PromptManager::new().await?);
    let config_file: Arc<ConfigFile> = Arc::new(ConfigFile::load().await?);
    let models_config = load_models_config().await?;

    for step_config in config.steps {
        match step_config.name.as_str() {
            "json_generation" => {
                engine.register_step(
                    step_config,
                    Arc::new(JsonGenerationStep {
                        prompt_manager: prompt_manager.clone(),
                        model_config: models_config.sentence_to_json.clone(),
                    }),
                );
            }
            "endpoint_matching" => {
                engine.register_step(
                    step_config,
                    Arc::new(EndpointMatchingStep {
                        config: config_file.clone(),
                    }),
                );
            }
            "field_matching" => {
                engine.register_step(step_config, Arc::new(FieldMatchingStep {}));
            }
            _ => {
                tracing::warn!("Unknown step: {}", step_config.name);
            }
        }
    }

    // Execute workflow
    engine.execute(sentence.to_string()).await
}
