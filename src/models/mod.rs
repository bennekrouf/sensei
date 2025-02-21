pub mod config;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub format: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct OllamaResponse {
    pub response: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Endpoint {
    pub id: String,
    pub text: String,
    pub description: String,
    pub parameters: Vec<EndpointParameter>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EndpointParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub alternatives: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    pub endpoints: Vec<Endpoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub description: String,
    pub semantic_value: Option<String>,
}
