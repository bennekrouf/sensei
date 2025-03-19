use crate::endpoint_client::endpoint::endpoint_service_client::EndpointServiceClient;
use crate::endpoint_client::endpoint::GetEndpointsRequest;
// src/analyze_sentence.rs
use crate::endpoint_client::{check_endpoint_service_health, convert_remote_endpoints};
use crate::models::config::load_models_config;
use crate::models::providers::ModelProvider;
use crate::models::ConfigFile;
use crate::models::EndpointParameter;
use crate::workflow::find_closest_endpoint::find_closest_endpoint;
use crate::workflow::match_fields::match_fields_semantic;
use crate::workflow::sentence_to_json::sentence_to_json;
use crate::workflow::WorkflowConfig;
use crate::workflow::WorkflowEngine;
use crate::workflow::WorkflowStep;
use serde_json::Value;
use std::error::Error;
use tonic::transport::Channel;
use tracing::{debug, error, info, warn};

pub struct AnalysisResult {
    pub json_output: Value,
    pub endpoint_id: String,
    pub endpoint_description: String,
    pub parameters: Vec<EndpointParameter>,
}

use async_trait::async_trait;
use std::sync::Arc;

// Step 2: Define each workflow step
pub struct ConfigurationLoadingStep {
    pub api_url: Option<String>,
    pub email: String,
}

#[async_trait]
impl WorkflowStep for ConfigurationLoadingStep {
    async fn execute(
        &self,
        context: &mut crate::workflow::context::WorkflowContext,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Loading configurations");

        // Set email in context
        context.email = Some(self.email.clone());

        // Flag to track if we've successfully loaded endpoints
        let mut endpoints_loaded = false;

        // Load endpoints configuration from remote API if URL is provided
        if let Some(api_url) = &self.api_url {
            info!("Loading endpoints from remote API: {}", api_url);

            // First verify the service is available, using same logic as server startup
            match check_endpoint_service_health(api_url).await {
                Ok(true) => {
                    info!("Remote endpoint service is available, fetching endpoints");

                    // Now attempt to fetch endpoints with increased timeout
                    let _client = reqwest::Client::builder()
                        .timeout(std::time::Duration::from_secs(10))
                        .build()
                        .unwrap_or_default();

                    // Try to create a channel with explicit timeout
                    match Channel::from_shared(api_url.clone()).map(|c| {
                        c.connect_timeout(std::time::Duration::from_secs(5))
                            .timeout(std::time::Duration::from_secs(10))
                    }) {
                        Ok(channel_builder) => {
                            match channel_builder.connect().await {
                                Ok(channel) => {
                                    // Create endpoint client with the connected channel
                                    let mut client = EndpointServiceClient::new(channel);

                                    // Make the request
                                    let request = tonic::Request::new(GetEndpointsRequest {
                                        email: self.email.clone(),
                                    });

                                    info!("Requesting endpoints for email: {}", self.email);

                                    // Attempt to get endpoints
                                    match client.get_endpoints(request).await {
                                        Ok(response) => {
                                            let mut stream = response.into_inner();
                                            let mut remote_endpoints = Vec::new();

                                            // Collect all endpoints from stream
                                            while let Some(resp) = stream.message().await? {
                                                info!(
                                                    "Received batch of {} endpoints",
                                                    resp.endpoints.len()
                                                );
                                                remote_endpoints.extend(resp.endpoints);
                                            }

                                            if remote_endpoints.is_empty() {
                                                warn!("Remote API returned empty endpoints list for email: {}", self.email);
                                            } else {
                                                info!("Successfully loaded {} endpoints for email: {}", 
                                                      remote_endpoints.len(), self.email);

                                                let endpoints =
                                                    convert_remote_endpoints(remote_endpoints);
                                                let config = ConfigFile { endpoints };
                                                context.endpoints_config = Some(config);
                                                endpoints_loaded = true;
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to get endpoints from service: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to connect to channel: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to create channel: {}", e);
                        }
                    }
                }
                Ok(false) => {
                    error!("Remote endpoint service at {} is not available", api_url);
                }
                Err(e) => {
                    error!("Error checking endpoint service health: {}", e);
                }
            }
        }

        // If we haven't loaded endpoints from API, try local file
        if !endpoints_loaded {
            info!("Attempting to load endpoints from local file");

            match tokio::fs::read_to_string("endpoints.yaml").await {
                Ok(config_str) => match serde_yaml::from_str::<ConfigFile>(&config_str) {
                    Ok(config) => {
                        info!(
                            "Successfully loaded {} endpoints from local file",
                            config.endpoints.len()
                        );
                        if config.endpoints.is_empty() {
                            return Err("Local endpoints file contains no endpoints".into());
                        }
                        context.endpoints_config = Some(config);
                        endpoints_loaded = true;
                    }
                    Err(e) => {
                        error!("Failed to parse local endpoints file: {}", e);
                        return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("No endpoint configuration available: Failed to parse endpoints.yaml: {}", e),
                            )));
                    }
                },
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        if self.api_url.is_some() {
                            error!("Local endpoints file not found and remote endpoint service unavailable");
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::NotFound,
                                format!("No endpoint configuration available: endpoints.yaml file not found and remote endpoint service unavailable for email: {}", self.email),
                            )));
                        } else {
                            error!("Local endpoints file not found: endpoints.yaml");
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::NotFound,
                                "No endpoint configuration available: endpoints.yaml file not found and no remote endpoint service configured",
                            )));
                        }
                    } else {
                        error!("Error reading local endpoints file: {}", e);
                        return Err(Box::new(std::io::Error::new(
                            e.kind(),
                            format!("No endpoint configuration available: Error reading endpoints.yaml: {}", e),
                        )));
                    }
                }
            }
        }

        // Verify that we have loaded endpoints
        if !endpoints_loaded {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No endpoints configuration available from remote service or local file for email: {}", self.email),
            )));
        }

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
        context: &mut crate::workflow::context::WorkflowContext,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Generating JSON from sentence");

        let json_result = sentence_to_json(&context.sentence, context.provider.clone()).await?;
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
        context: &mut crate::workflow::context::WorkflowContext,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Finding closest matching endpoint");

        let config = context
            .endpoints_config
            .as_ref()
            .ok_or("Endpoints configuration not loaded")?;

        let endpoint_result =
            find_closest_endpoint(config, &context.sentence, context.provider.clone()).await?;
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
        context: &mut crate::workflow::context::WorkflowContext,
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

        let semantic_results =
            match_fields_semantic(json_output, endpoint, context.provider.clone()).await?;

        // Convert semantic results to parameters
        let parameters: Vec<EndpointParameter> = endpoint
            .parameters
            .iter()
            .map(|param| {
                let semantic_value = semantic_results
                    .iter()
                    .find(|(name, _, _)| name == &param.name)
                    .and_then(|(_, _, value)| value.clone());

                EndpointParameter {
                    name: param.name.clone(),
                    description: param.description.clone(),
                    semantic_value,
                    alternatives: param.alternatives.clone(),
                    required: param.required,
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

// Step 3: Workflow Configuration (unchanged)
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

// Step 4: Updated analyze_sentence function with API URL parameter
pub async fn analyze_sentence(
    sentence: &str,
    provider: Arc<dyn ModelProvider>,
    api_url: Option<String>,
    email: &str,
) -> Result<AnalysisResult, Box<dyn Error + Send + Sync>> {
    info!("Starting sentence analysis for: {}", sentence);
    info!("Using email: {}", email);

    if let Some(url) = &api_url {
        info!("Using remote API for endpoints: {}", url);
    } else {
        info!("Using local endpoints file");
    }

    // Initialize workflow engine
    let config: WorkflowConfig = serde_yaml::from_str(WORKFLOW_CONFIG)?;
    let mut engine = WorkflowEngine::new();

    // Register all steps
    for step_config in config.steps {
        match step_config.name.as_str() {
            "configuration_loading" => {
                engine.register_step(
                    step_config,
                    Arc::new(ConfigurationLoadingStep {
                        api_url: api_url.clone(),
                        email: email.to_string(),
                    }),
                );
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
    let context = engine.execute(sentence.to_string(), provider).await?;

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
