use super::{
    find_closest_endpoint::find_closest_endpoint, match_fields::match_fields_semantic,
    sentence_to_json::sentence_to_json,
};
use crate::models::ConfigFile;
use crate::workflow::context::WorkflowContext;
use std::{error::Error, sync::Arc};

pub struct JsonGenerationStep {}

// Trait defining a workflow step
#[async_trait]
pub trait WorkflowStep: Send + Sync {
    async fn execute(
        &self,
        context: &mut WorkflowContext,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
    fn name(&self) -> &'static str;
}

#[async_trait]
impl WorkflowStep for JsonGenerationStep {
    async fn execute(
        &self,
        context: &mut WorkflowContext,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Pass both the sentence and the provider from context
        let json_output = sentence_to_json(&context.sentence, context.provider.clone()).await?;
        context.json_output = Some(json_output);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "json_generation"
    }
}

pub struct EndpointMatchingStep {
    pub config: Arc<ConfigFile>,
}

#[async_trait]
impl WorkflowStep for EndpointMatchingStep {
    async fn execute(
        &self,
        context: &mut WorkflowContext,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let endpoint =
            find_closest_endpoint(&self.config, &context.sentence, context.provider.clone())
                .await?;
        context.matched_endpoint = Some(endpoint);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "endpoint_matching"
    }
}

use crate::models::EndpointParameter;
use async_trait::async_trait;

// Workflow configuration loaded from YAML
pub struct FieldMatchingStep {}

#[async_trait]
impl WorkflowStep for FieldMatchingStep {
    async fn execute(
        &self,
        context: &mut WorkflowContext,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let (Some(json), Some(endpoint)) = (&context.json_output, &context.matched_endpoint) {
            let semantic_results =
                match_fields_semantic(json, endpoint, context.provider.clone()).await?;

            // Convert tuple results into Parameter structs
            let parameters = semantic_results
                .into_iter()
                .map(|(name, description, semantic_value)| EndpointParameter {
                    name,
                    description,
                    semantic_value,
                    alternatives: None,
                    required: None,
                })
                .collect();

            context.parameters = parameters;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "field_matching"
    }
}
