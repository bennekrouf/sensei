use super::config::RetryConfig;
use super::{config::StepConfig, steps::WorkflowStep, WorkflowContext};
use std::error::Error;
use std::sync::Arc;

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
