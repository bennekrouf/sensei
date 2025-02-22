use crate::models::{config::ModelsConfig, ConfigFile, Endpoint, EndpointParameter};
use serde_json::Value;

#[derive(Clone, Debug, Default)]
pub struct WorkflowContext {
    // Input
    pub sentence: String,

    // Configurations
    pub models_config: Option<ModelsConfig>,
    pub endpoints_config: Option<ConfigFile>,

    // Processing state
    pub json_output: Option<Value>,
    pub matched_endpoint: Option<Endpoint>,
    pub parameters: Vec<EndpointParameter>,
    pub endpoint_id: Option<String>,
    pub endpoint_description: Option<String>,
}
