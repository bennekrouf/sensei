use tonic::{Request, Response, Status};
use tonic::metadata::MetadataMap;
use futures::Stream;
use tokio::sync::mpsc;

use std::pin::Pin;
pub mod sentence {
    tonic::include_proto!("sentence");
}

use sentence::sentence_service_server::SentenceService;
use sentence::{SentenceRequest, SentenceResponse, Parameter};
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;

#[derive(Debug, Default)]
pub struct SentenceAnalyzeService;

impl SentenceAnalyzeService {
    // Helper function to extract client_id from metadata
    fn get_client_id(metadata: &MetadataMap) -> String {
        metadata
            .get("client-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown-client")
            .to_string()
    }
}

#[tonic::async_trait]
impl SentenceService for SentenceAnalyzeService {

    type AnalyzeSentenceStream = Pin<Box<dyn Stream<Item = Result<SentenceResponse, Status>> + Send>>;

    #[tracing::instrument(skip(self, request), fields(client_id))]
    async fn analyze_sentence(
        &self,
        request: Request<SentenceRequest>,
    ) -> Result<Response<Self::AnalyzeSentenceStream>, Status> {
        println!("fjdskljhfjfkl jgfkldjgfdhgkl !!!!");

        let (tx, rx) = mpsc::channel(10);

        tokio::spawn(async move {
            for i in 0..5 {
                let response = SentenceResponse {
                    endpoint_id: format!("test-endpoint-{}", i),
                    endpoint_description: "Test endpoint".to_string(),
                    parameters: vec![
                        Parameter {
                            name: "test".to_string(),
                            description: "test param".to_string(),
                            value: Some(format!("test value {}", i)),
                        }
                    ],
                    json_output: format!("{{\"status\": \"ok\", \"iteration\": {}}}", i),
                };

                if tx.send(Ok(response)).await.is_err() {
                    break;
                }

                // tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))

        // // Log the full request details
        // info!("Request metadata: {:?}", request.metadata());
        // info!("Request headers: {:?}", request.metadata().keys());
        //
        // let client_id = Self::get_client_id(request.metadata());
        //
        // // Add debug logging for request details
        // tracing::debug!(
        //     "Full request details: {:?}",
        //     request.metadata().iter()
        //         .map(|item| match item {
        //             tonic::metadata::KeyAndValueRef::Ascii(k, v) => 
        //                 (k.as_str(), v.to_str().unwrap_or("invalid")),
        //             tonic::metadata::KeyAndValueRef::Binary(k, _) => 
        //                 (k.as_str(), "binary value")
        //         })
        //         .collect::<Vec<_>>()
        // );
        //
        // let sentence = request.into_inner().sentence;
        // info!(sentence = %sentence, "Processing sentence request");
        //
        // let analyze_span = tracing::info_span!("analyze_sentence", client_id = %client_id);
        //
        // // Process the request and create response
        // let result = analyze_sentence(&sentence)
        //     .instrument(analyze_span)
        //     .await;
        //
        // match result {
        //     Ok(result) => {
        //         info!(client_id = %client_id, "Analysis completed");
        //
        //         let response = SentenceResponse {
        //             endpoint_id: result.endpoint_id,
        //             endpoint_description: result.endpoint_description,
        //             parameters: result.parameters
        //                 .into_iter()
        //                 .map(|(name, description, value)| Parameter {
        //                     name,
        //                     description,
        //                     value,
        //                 })
        //                 .collect(),
        //             json_output: serde_json::to_string(&result.json_output)
        //                 .map_err(|e| Status::internal(format!("JSON serialization failed: {}", e)))?,
        //         };
        //
        //         info!(
        //             client_id = %client_id,
        //             response = ?response,
        //             "Sending response"
        //         );
        //
        //         Ok(Response::new(response))
        //     }
        //     Err(e) => {
        //         error!(
        //             sentence = %sentence,
        //             error = %e,
        //             client_id = %client_id,
        //             "Analysis failed"
        //         );
        //         Err(Status::internal(format!("Analysis failed: {}", e)))
        //     }
        // }
    }
}
