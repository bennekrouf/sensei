use std::error::Error;

use tracing::{debug, error, info};

pub async fn extract_matched_action(ollama_response: &str) -> Result<String, Box<dyn Error>> {
    debug!("Extracting matched action from response");

    // Get the last non-empty line from the response
    let last_line = ollama_response
        .lines()
        .filter(|line| !line.trim().is_empty())
        .last()
        .ok_or_else(|| {
            error!("No valid lines found in response");
            "Empty response"
        })?;

    debug!("Extracted last line: '{}'", last_line);

    // Remove any single or double quotes that might be present
    let cleaned_response = last_line
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string();

    if cleaned_response.is_empty() {
        error!("Extracted response is empty after cleaning");
        return Err("Empty extracted response".into());
    }

    debug!("Final cleaned response: '{}'", cleaned_response);
    Ok(cleaned_response)
}
