use crate::models::providers::ModelProvider;
use crate::sentence_service::sentence::sentence_service_server::SentenceServiceServer;
use crate::sentence_service::SentenceAnalyzeService;
use std::sync::Arc;
use tonic::transport::Server;
use tonic_reflection::server::Builder;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

pub async fn start_sentence_grpc_server(
    provider: Arc<dyn ModelProvider>,
    api_url: Option<String>,
    default_email: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "0.0.0.0:50053".parse()?;
    info!("Starting sentence analysis gRPC server on {}", addr);

    if let Some(url) = &api_url {
        info!("Using remote endpoint API at: {}", url);
    } else {
        info!("Using local endpoints file");
    }

    let descriptor_set = include_bytes!(concat!(env!("OUT_DIR"), "/sentence_descriptor.bin"));
    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(descriptor_set)
        .build_v1()?;

    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any)
        .expose_headers(Any);

    tracing::info!("Starting semantic gRPC server on {}", addr);

    // Use the provider that was passed in from main.rs
    let sentence_service = SentenceAnalyzeService::new(provider, api_url, default_email);
    let service = SentenceServiceServer::new(sentence_service);

    match Server::builder()
        .accept_http1(true)
        .max_concurrent_streams(128) // Set reasonable limits
        .tcp_keepalive(Some(std::time::Duration::from_secs(60)))
        .tcp_nodelay(true)
        .layer(cors) // Add CORS layer
        .layer(GrpcWebLayer::new())
        .add_service(service)
        .add_service(reflection_service) // Add reflection service
        .serve_with_shutdown(addr, async {
            tokio::signal::ctrl_c().await.ok();
            info!("Shutting down semantic server...");
        })
        .await
    {
        Ok(_) => Ok::<(), Box<dyn std::error::Error + Send + Sync>>(()),
        Err(e) => {
            if e.to_string().contains("Address already in use") {
                tracing::error!("Port already in use. Please stop other instances first.");
            }
            Err(e.into())
        }
    }
}
