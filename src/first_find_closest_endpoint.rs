use crate::models::Endpoint;
use crate::second_extract_matched_action::extract_matched_action;
use crate::{
    call_ollama::call_ollama, third_find_endpoint_by_substring::find_endpoint_by_substring,
};

use crate::models::ConfigFile;
use std::error::Error;

use crate::prompts::PromptManager;
use tracing::{debug, error, info};

pub async fn find_closest_endpoint(
    config: &ConfigFile,
    input_sentence: &str,
    model: &str,
) -> Result<Endpoint, Box<dyn Error + Send + Sync>> {
    info!("Starting endpoint matching for input: {}", input_sentence);
    debug!("Available endpoints: {}", config.endpoints.len());

    // Initialize the PromptManager
    let prompt_manager = PromptManager::new().await?;

    // Generate the actions list
    let actions_list = config
        .endpoints
        .iter()
        .map(|e| format!("- {}", e.text))
        .collect::<Vec<String>>()
        .join("\n");

    // Get formatted prompt from PromptManager
    let prompt = prompt_manager.format_find_endpoint(input_sentence, &actions_list);

    debug!("Generated prompt:\n{}", prompt);

    // Call Ollama
    info!("Calling Ollama with model: {}", model);
    let raw_response = call_ollama(model, &prompt).await?;
    debug!("Raw Ollama response: '{}'", raw_response);

    let cleaned_response = extract_matched_action(&raw_response).await?;
    info!("Raw cleaned_response response: '{}'", cleaned_response);

    let matched_endpoint = match find_endpoint_by_substring(config, &cleaned_response) {
        Ok(endpoint) => endpoint.clone(),
        Err(_) => {
            error!("No endpoint matched the response: '{}'", cleaned_response);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No matching endpoint found",
            )));
        }
    };

    info!("Found matching endpoint: {}", matched_endpoint.id);
    Ok(matched_endpoint)
}
