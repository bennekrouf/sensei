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
        ConfigFile, Parameter,
    },
    prompts::PromptManager,
    zero_sentence_to_json::sentence_to_json,
};

// Trait defining a workflow step
#[async_trait]
pub trait WorkflowStep: Send + Sync {
    async fn execute(
        &self,
        context: &mut WorkflowContext,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
    fn name(&self) -> &'static str;
}

// Workflow configuration loaded from YAML
#[derive(Debug, Deserialize)]
pub struct WorkflowConfig {
    pub steps: Vec<StepConfig>,
}

#[derive(Debug, Deserialize)]
pub struct StepConfig {
    pub name: String,
    pub enabled: bool,
    pub retry: Option<RetryConfig>,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub delay_ms: u64,
}

// Workflow engine
pub struct WorkflowEngine {
    steps: Vec<(StepConfig, Arc<dyn WorkflowStep>)>,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn register_step(&mut self, config: StepConfig, step: Arc<dyn WorkflowStep>) {
        self.steps.push((config, step));
    }

    pub async fn execute(
        &self,
        input: String,
    ) -> Result<WorkflowContext, Box<dyn Error + Send + Sync>> {
        let mut context = WorkflowContext {
            sentence: input,
            ..Default::default()
        };

        for (config, step) in &self.steps {
            if !config.enabled {
                continue;
            }

            tracing::info!("Executing step: {}", step.name());

            let result = match &config.retry {
                Some(retry) => {
                    self.execute_with_retry(step.as_ref(), &mut context, retry)
                        .await
                }
                None => step.execute(&mut context).await,
            };

            if let Err(e) = result {
                tracing::error!("Step {} failed: {}", step.name(), e);
                return Err(e);
            }
        }

        Ok(context)
    }

    async fn execute_with_retry(
        &self,
        step: &dyn WorkflowStep,
        context: &mut WorkflowContext,
        retry: &RetryConfig,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut attempts = 0;
        loop {
            match step.execute(context).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    attempts += 1;
                    if attempts >= retry.max_attempts {
                        return Err(e);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(retry.delay_ms)).await;
                }
            }
        }
    }
}

// Example implementation of workflow steps
pub struct JsonGenerationStep {
    pub prompt_manager: Arc<PromptManager>,
    pub model_config: ModelConfig,
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
                .map(|(name, description, semantic_value)| Parameter {
                    name,
                    description,
                    semantic_value,
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
