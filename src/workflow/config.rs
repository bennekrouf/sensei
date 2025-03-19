use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WorkflowConfig {
    pub steps: Vec<StepConfig>,
}

#[derive(Debug, Deserialize)]
pub struct StepConfig {
    pub name: String,
    pub enabled: bool,
    pub retry: Option<RetryConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub delay_ms: u64,
}
