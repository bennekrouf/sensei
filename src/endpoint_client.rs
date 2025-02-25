// src/endpoint_client.rs
use tonic::transport::Channel;
use std::error::Error;
use tracing::{info, error};

pub mod endpoint {
    tonic::include_proto!("endpoint");
}

use endpoint::endpoint_service_client::EndpointServiceClient;
use endpoint::{GetEndpointsRequest, Endpoint};

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
    
    info!("Successfully fetched {} endpoints from remote service", endpoints.len());
    
    Ok(endpoints)
}

// Convert gRPC Endpoint to our internal Endpoint structure
pub fn convert_remote_endpoints(
    remote_endpoints: Vec<endpoint::Endpoint>
) -> Vec<crate::models::Endpoint> {
    remote_endpoints
        .into_iter()
        .map(|re| crate::models::Endpoint {
            id: re.id,
            text: re.text,
            description: re.description,
            parameters: re.parameters
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
