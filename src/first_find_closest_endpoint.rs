use crate::models::Endpoint;
use crate::second_extract_matched_action::extract_matched_action;
use crate::{
    call_ollama::call_ollama, third_find_endpoint_by_substring::find_endpoint_by_substring,
};

use crate::models::ConfigFile;
use std::error::Error;

use tracing::{debug, error, info};

pub async fn find_closest_endpoint(
    config: &ConfigFile,
    input_sentence: &str,
    model: &str,
) -> Result<Endpoint, Box<dyn Error>> {
    info!("Starting endpoint matching for input: {}", input_sentence);
    debug!("Available endpoints: {}", config.endpoints.len());

    let prompt = format!(
        r#"Given this reference sentence: '{}'
Compare it to these possible actions and identify which one most closely matches the core intent and meaning of the reference sentence:
{}
Determine the closest match by:
1. Identifying the main verb/action in the reference sentence
2. Extracting key elements (who, what, when, where, why, how)
3. Comparing these elements to the fundamental purpose of each action option
4. Selecting the action that best captures the essential meaning and purpose
Only output the exact text of the single best matching action from the list, nothing else."#,
        input_sentence,
        config
            .endpoints
            .iter()
            .map(|e| format!("- {}", e.text))
            .collect::<Vec<String>>()
            .join("\n")
    );

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
