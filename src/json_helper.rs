use regex::Regex;
use serde_json::Value;
use std::error::Error;
use tracing::{debug, error};

pub fn sanitize_json(raw_text: &str) -> Result<Value, Box<dyn Error + Send + Sync>> {
    //debug!("Sanitizing JSON from raw text:\n{}", raw_text);

    // Extract JSON using regex
    let re = Regex::new(r"\{[\s\S]*\}")?;
    let json_str = re
        .find(raw_text)
        .ok_or_else(|| {
            error!("No JSON found in response: {}", raw_text);
            "No JSON structure found in response"
        })?
        .as_str();

    debug!("Extracted JSON string:\n{}", json_str);

    // Parse the JSON
    let parsed_json: Value = serde_json::from_str(json_str).map_err(|e| {
        error!("Failed to parse JSON: {}\nRaw JSON string: {}", e, json_str);
        format!("Failed to parse JSON: {}. Raw JSON: {}", e, json_str)
    })?;

    debug!("Successfully parsed JSON");
    Ok(parsed_json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_valid_json() {
        let input = r#"Some text before {"key": "value"} some text after"#;
        let result = sanitize_json(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["key"], "value");
    }

    #[test]
    fn test_sanitize_invalid_input() {
        let input = "No JSON here";
        let result = sanitize_json(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_complex_json() {
        let input = r#"Here's the output: {
            "nested": {
                "array": [1, 2, 3],
                "object": {"key": "value"}
            }
        } and some text after"#;
        let result = sanitize_json(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["nested"]["array"][0], 1);
    }
}
