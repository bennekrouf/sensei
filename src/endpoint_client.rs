// src/endpoint_client.rs
use crate::models::config::load_endpoint_client_config;
use std::error::Error;
use tonic::transport::Channel;
use tracing::{error, info, warn};

pub mod endpoint {
    tonic::include_proto!("endpoint");
}

use endpoint::endpoint_service_client::EndpointServiceClient;
use endpoint::{Endpoint, GetEndpointsRequest};

/// Fetch endpoints from remote gRPC service
pub async fn fetch_remote_endpoints(
    addr: String,
    email: &str,
) -> Result<Vec<Endpoint>, Box<dyn Error + Send + Sync>> {
    info!("Connecting to remote endpoint service at {}", addr);

    // Create a channel to the server
    let channel = match Channel::from_shared(addr) {
        Ok(channel) => channel.connect().await?,
        Err(e) => {
            error!("Failed to create channel: {}", e);
            return Err(Box::new(e));
        }
    };

    // Create the gRPC client
    let mut client = EndpointServiceClient::new(channel);

    // Prepare the request
    let request = tonic::Request::new(GetEndpointsRequest {
        email: email.to_string(),
    });

    info!("Fetching endpoints for email: {}", email);

    // Make the streaming call
    let mut stream = client.get_default_endpoints(request).await?.into_inner();

    // Collect all endpoints from the stream
    let mut endpoints = Vec::new();

    while let Some(response) = stream.message().await? {
        info!("Received batch of {} endpoints", response.endpoints.len());
        endpoints.extend(response.endpoints);
    }

    info!(
        "Successfully fetched {} endpoints from remote service",
        endpoints.len()
    );

    Ok(endpoints)
}

/// Get the default API URL from configuration if not provided via CLI
pub async fn get_default_api_url() -> Result<String, Box<dyn Error + Send + Sync>> {
    let endpoint_client_config = load_endpoint_client_config().await?;
    Ok(endpoint_client_config.default_address)
}

// Convert gRPC Endpoint to our internal Endpoint structure
pub fn convert_remote_endpoints(
    remote_endpoints: Vec<endpoint::Endpoint>,
) -> Vec<crate::models::Endpoint> {
    remote_endpoints
        .into_iter()
        .map(|re| crate::models::Endpoint {
            id: re.id,
            text: re.text,
            description: re.description,
            parameters: re
                .parameters
                .into_iter()
                .map(|rp| crate::models::EndpointParameter {
                    name: rp.name,
                    description: rp.description,
                    required: Some(rp.required),
                    alternatives: Some(rp.alternatives),
                    semantic_value: None,
                })
                .collect(),
        })
        .collect()
}

pub async fn check_endpoint_service_health(
    addr: &str,
) -> Result<bool, Box<dyn Error + Send + Sync>> {
    info!("Checking health of endpoint service at {}", addr);

    // Try to create a channel to the server
    match Channel::from_shared(addr.to_string()) {
        Ok(channel) => match channel.connect().await {
            Ok(_) => {
                info!("Endpoint service is available at {}", addr);
                Ok(true)
            }
            Err(e) => {
                warn!("Endpoint service is not available at {}: {}", addr, e);
                Ok(false)
            }
        },
        Err(e) => {
            error!("Invalid endpoint service address {}: {}", addr, e);
            Err(Box::new(e))
        }
    }
}

/// Check if endpoints are properly configured
pub async fn verify_endpoints_configuration(
    api_url: Option<String>,
) -> Result<bool, Box<dyn Error + Send + Sync>> {
    // First check if remote API is available
    if let Some(url) = &api_url {
        match check_endpoint_service_health(url).await {
            Ok(true) => return Ok(true),
            Ok(false) => {
                info!("Remote endpoint service unavailable, checking for local endpoints file");
            }
            Err(e) => {
                warn!("Error checking endpoint service: {}", e);
                // Continue to check local file
            }
        }
    }

    // Then check if local file exists
    match tokio::fs::metadata("endpoints.yaml").await {
        Ok(metadata) => {
            if metadata.is_file() {
                info!("Local endpoints file exists");
                return Ok(true);
            } else {
                warn!("endpoints.yaml exists but is not a file");
                return Ok(false);
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                error!("No local endpoints file found and no remote service available");
                return Ok(false);
            } else {
                error!("Error checking for local endpoints file: {}", e);
                return Err(Box::new(e));
            }
        }
    }
}
