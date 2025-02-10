use crate::analyze_sentence::analyze_sentence;
use tonic::{Request, Response, Status};
use tonic_reflection::server::Builder as ReflectionBuilder;
use tracing::{error, info};
use tracing_futures::Instrument;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{Any, CorsLayer};

pub mod sentence {
    tonic::include_proto!("sentence");
}

use sentence::sentence_service_server::{SentenceService, SentenceServiceServer};
use sentence::{SentenceRequest, SentenceResponse, Parameter};

#[derive(Debug, Default)]
pub struct SentenceAnalyzeService;

#[tonic::async_trait]
impl SentenceService for SentenceAnalyzeService {
    async fn analyze_sentence(
        &self,
        request: Request<SentenceRequest>,
    ) -> Result<Response<SentenceResponse>, Status> {
        let sentence = request.into_inner().sentence;
        info!(sentence = %sentence, "Received sentence analysis request");

        match analyze_sentence(&sentence).await {
            Ok(result) => {
                let parameters = result.parameters
                    .into_iter()
                    .map(|(name, description, value)| Parameter {
                        name,
                        description,
                        value,
                    })
                    .collect();

                let json_output = serde_json::to_string(&result.json_output)
                    .map_err(|e| Status::internal(format!("Failed to serialize JSON: {}", e)))?;

                Ok(Response::new(SentenceResponse {
                    endpoint_id: result.endpoint_id,
                    endpoint_description: result.endpoint_description,
                    parameters,
                    json_output,
                }))
            }
            Err(e) => {
                error!(sentence = %sentence, error = %e, "Sentence analysis failed");
                Err(Status::internal(format!("Analysis failed: {}", e)))
            }
        }
    }
}

pub async fn start_sentence_grpc_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "0.0.0.0:50053".parse()?;
    info!(address = %addr, "Starting sentence analysis gRPC server");

    let sentence_service = SentenceAnalyzeService::default();

    let descriptor_set = include_bytes!(concat!(env!("OUT_DIR"), "/sentence_descriptor.bin"));
    let reflection_service = ReflectionBuilder::configure()
        .register_encoded_file_descriptor_set(descriptor_set)
        .build_v1()?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    Server::builder()
        .accept_http1(true)
        .layer(cors)
        .layer(GrpcWebLayer::new())
        .add_service(SentenceServiceServer::new(sentence_service))
        .add_service(reflection_service)
        .serve(addr)
        .instrument(tracing::info_span!("sentence_grpc_server", addr = %addr))
        .await?;

    info!("Sentence analysis gRPC server has been shut down");
    Ok(())
}
