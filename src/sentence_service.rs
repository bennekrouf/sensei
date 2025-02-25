use crate::analyze_sentence::analyze_sentence;
use crate::models::providers::ModelProvider;
use futures::Stream;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::metadata::MetadataMap;
use tonic::{Request, Response, Status};

pub mod sentence {
    tonic::include_proto!("sentence");
}

use sentence::sentence_service_server::SentenceService;
use sentence::{Parameter, SentenceRequest, SentenceResponse};
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tracing::Instrument;

pub struct SentenceAnalyzeService {
    provider: Arc<dyn ModelProvider>,
}

impl SentenceAnalyzeService {
    // Add a constructor to store the provider
    pub fn new(provider: Arc<dyn ModelProvider>) -> Self {
        Self { provider }
    }

    // For backward compatibility, but not recommended
    pub fn default() -> Self {
        panic!("SentenceAnalyzeService::default() is not supported. Use SentenceAnalyzeService::new() instead.");
    }

    // Helper function to extract client_id from metadata
    fn get_client_id(metadata: &MetadataMap) -> String {
        metadata
            .get("client-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown-client")
            .to_string()
    }
}

// Implement Debug manually
impl std::fmt::Debug for SentenceAnalyzeService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SentenceAnalyzeService")
            .field("provider", &"<dyn ModelProvider>")
            .finish()
    }
}

#[tonic::async_trait]
impl SentenceService for SentenceAnalyzeService {
    type AnalyzeSentenceStream =
        Pin<Box<dyn Stream<Item = Result<SentenceResponse, Status>> + Send>>;

    #[tracing::instrument(skip(self, request), fields(client_id))]
    async fn analyze_sentence(
        &self,
        request: Request<SentenceRequest>,
    ) -> Result<Response<Self::AnalyzeSentenceStream>, Status> {
        let metadata = request.metadata().clone();
        // Log request details
        tracing::info!("Request metadata: {:?}", metadata);
        tracing::info!("Request headers: {:?}", metadata.keys());

        let client_id = Self::get_client_id(&metadata);
        let sentence = request.into_inner().sentence;
        tracing::info!(sentence = %sentence, "Processing sentence request");

        // Debug logging for request details
        tracing::debug!(
            "Full request details: {:?}",
            metadata
                .iter()
                .map(|item| match item {
                    tonic::metadata::KeyAndValueRef::Ascii(k, v) =>
                        (k.as_str(), v.to_str().unwrap_or("invalid")),
                    tonic::metadata::KeyAndValueRef::Binary(k, _) => (k.as_str(), "binary value"),
                })
                .collect::<Vec<_>>()
        );

        let (tx, rx) = mpsc::channel(10);
        let analyze_span = tracing::info_span!("analyze_sentence", client_id = %client_id);

        // Clone the provider to move into the spawned task
        let provider_clone = self.provider.clone();

        tokio::spawn(async move {
            // Pass both the sentence and the provider to analyze_sentence
            let result = analyze_sentence(&sentence, provider_clone)
                .instrument(analyze_span)
                .await;

            match result {
                Ok(result) => {
                    tracing::info!(client_id = %client_id, "Analysis completed");

                    let response = SentenceResponse {
                        endpoint_id: result.endpoint_id,
                        endpoint_description: result.endpoint_description,
                        parameters: result
                            .parameters
                            .into_iter()
                            .map(|param| Parameter {
                                name: param.name,
                                description: param.description,
                                semantic_value: param.semantic_value,
                            })
                            .collect(),
                        json_output: match serde_json::to_string(&result.json_output) {
                            Ok(json) => json,
                            Err(e) => {
                                tracing::error!(error = %e, "JSON serialization failed");
                                format!("{{\"error\": \"JSON serialization failed: {}\"}}", e)
                            }
                        },
                    };

                    tracing::info!(
                        client_id = %client_id,
                        response = ?response,
                        "Sending response"
                    );

                    if tx.send(Ok(response)).await.is_err() {
                        tracing::error!(client_id = %client_id, "Failed to send response - stream closed");
                    }
                }
                Err(e) => {
                    tracing::error!(
                        sentence = %sentence,
                        error = %e,
                        client_id = %client_id,
                        "Analysis failed"
                    );

                    let _ = tx
                        .send(Err(Status::internal(format!("Analysis failed: {}", e))))
                        .await;
                }
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }
}
