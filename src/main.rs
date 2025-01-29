mod models;

use crate::models::Endpoint;
use models::ConfigFile;
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
    let response = call_ollama(model, &prompt).await?;
    debug!("Raw Ollama response: '{}'", response);

    // Log available endpoints for comparison
    debug!("Available endpoint texts:");
    for endpoint in &config.endpoints {
        debug!(
            "- '{}' (id: {})",
            endpoint.text.trim().to_lowercase(),
            endpoint.id
        );
    }
    //debug!(
    //    "Looking for match with response: '{}'",
    //    response.trim().to_lowercase()
    //);

    // Find the matching endpoint with detailed logging
    let matched_endpoint = config
        .endpoints
        .iter()
        .find(|e| {
            let endpoint_text = e.text.trim().to_lowercase();
            let response_text = response.trim().to_lowercase();
            let matches = endpoint_text == response_text;
            //debug!(
            //    "Comparing endpoint '{}' with response '{}': {}",
            //    endpoint_text, response_text, matches
            //);
            matches
        })
        .ok_or_else(|| {
            error!(
                "No endpoint matched",
                //response.trim().to_lowercase()
            );
            "No matching endpoint found"
        })?;

    info!("Found matching endpoint: {}", matched_endpoint.id);
    Ok(matched_endpoint.clone())
}

pub async fn call_ollama(model: &str, prompt: &str) -> Result<String, Box<dyn Error>> {
    debug!("Creating Ollama request for model: {}", model);
    let client = reqwest::Client::new();
    let request_body = GenerateRequest {
        model: model.to_string(),
        prompt: prompt.to_string(),
        stream: false,
        format: None, // Remove JSON format as we want plain text
    };

    info!("Sending request to Ollama");
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request_body)
        .send()
        .await?;

    debug!("Response status: {}", response.status());

    // Check if the response is successful
    if !response.status().is_success() {
        error!("Ollama request failed with status: {}", response.status());
        return Err(format!("Request failed with status: {}", response.status()).into());
    }

    let response_obj = response.json::<OllamaResponse>().await?;

    // Add validation for empty response
    if response_obj.response.trim().is_empty() {
        error!("Received empty response from Ollama");
        return Err("Empty response from Ollama".into());
    }

    //debug!("Parsed response: '{}'", response_obj.response.trim());

    Ok(response_obj.response.trim().to_owned())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("Starting application");

    // Test Ollama connection before proceeding
    //test_ollama_connection("deepseek-r1:8b").await?;

    example_usage().await?;
    Ok(())
}

pub async fn example_usage() -> Result<(), Box<dyn Error>> {
    info!("Loading configuration file");
    let config_str = tokio::fs::read_to_string("config.yaml").await?;
    debug!("Config file content length: {}", config_str.len());

    let config: ConfigFile = serde_yaml::from_str(&config_str)?;
    info!(
        "Configuration loaded with {} endpoints",
        config.endpoints.len()
    );

    let input =
        "schedule a meeting tomorrow at 2pm for 1 hour with Salem Mejid to discuss project status";
    info!("Processing input: {}", input);

    let result = find_closest_endpoint(&config, input, "deepseek-r1:8b").await?;

    println!("Matched endpoint: {}", result.id);
    println!("Description: {}", result.description);
    println!("\nRequired parameters:");
    for param in result.parameters.iter().filter(|p| p.required) {
        println!("- {}: {}", param.name, param.description);
    }

    Ok(())
}

//pub async fn test_ollama_connection(model: &str) -> Result<(), Box<dyn Error>> {
//    info!("Testing Ollama connection with model: {}", model);
//
//    let test_prompt = "Say 'test' and nothing else";
//    match call_ollama(model, test_prompt).await {
//        Ok(response) => {
//            info!(
//                "Successfully connected to Ollama. Test response: '{}'",
//                response
//            );
//            Ok(())
//        }
//        Err(e) => {
//            error!("Failed to connect to Ollama: {}", e);
//            Err(e)
//        }
//    }
//}

//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    #[tokio::test]
//    async fn test_find_closest_endpoint() {
//        let config_str = r#"
//endpoints:
//  - id: "schedule_meeting"
//    text: "schedule meeting"
//    description: "Schedule a meeting or appointment"
//    parameters: []
//  - id: "send_email"
//    text: "send email"
//    description: "Send an email with possible attachments"
//    parameters: []
//"#;
//        let config: ConfigFile = serde_yaml::from_str(config_str).unwrap();
//
//        let input = "schedule a meeting tomorrow at 2pm";
//        let result = find_closest_endpoint(&config, input, "deepseek-r1:8b")
//            .await
//            .unwrap();
//
//        assert_eq!(result.id, "schedule_meeting");
//    }
//}
use crate::models::GenerateRequest;
use crate::models::OllamaResponse;
