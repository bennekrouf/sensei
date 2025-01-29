use crate::models::{ConfigFile, Endpoint};
use std::error::Error;
use tracing::{debug, error};

// Finds the best matching endpoint using substring matching
pub fn find_endpoint_by_substring<'a>(
    config: &'a ConfigFile,
    response: &str,
) -> Result<&'a Endpoint, Box<dyn Error>> {
    let response_lower = response.to_lowercase();
    debug!(
        "Attempting substring matching with response: '{}'",
        response_lower
    );

    // Find all endpoints that might match
    let matches: Vec<_> = config
        .endpoints
        .iter()
        .filter(|endpoint| {
            let endpoint_text = endpoint.text.trim().to_lowercase();

            // Try different matching strategies
            response_lower.contains(&endpoint_text) || // Response contains endpoint text
            endpoint_text.split_whitespace().all(|word| response_lower.contains(word))
            // All words in endpoint are in response
        })
        .collect();

    debug!("Found {} potential matches", matches.len());

    // Log all matches for debugging
    for (i, endpoint) in matches.iter().enumerate() {
        debug!(
            "Match candidate {}: '{}' (id: {})",
            i + 1,
            endpoint.text,
            endpoint.id
        );
    }

    // Take the first match if available
    matches
        .first()
        .ok_or_else(|| {
            let error_msg = format!("No endpoint matched the response: '{}'", response);
            error!("{}", error_msg);
            Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, error_msg)) as Box<dyn Error>
        })
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ConfigFile, Endpoint};

    fn create_test_config() -> ConfigFile {
        ConfigFile {
            endpoints: vec![
                Endpoint {
                    id: "schedule_meeting".to_string(),
                    text: "schedule meeting".to_string(),
                    description: "Schedule a meeting".to_string(),
                    parameters: vec![],
                },
                // Add more test endpoints as needed
            ],
        }
    }

    #[test]
    fn test_substring_matching() {
        let config = create_test_config();

        // Test cases that should match
        let test_cases = vec![
            "**Answer:** schedule meeting",
            "The answer is: schedule meeting",
            "schedule meeting is the best match",
            "We should schedule meeting",
            "'schedule meeting'",
            "schedule  meeting", // Extra spaces
        ];

        for case in test_cases {
            let result = find_endpoint_by_substring(&config, case);
            assert!(result.is_ok(), "Failed to match: {}", case);
            assert_eq!(result.unwrap().id, "schedule meeting");
        }

        // Test cases that should not match
        let negative_cases = vec!["something completely different", "scheduled meetings", ""];

        for case in negative_cases {
            let result = find_endpoint_by_substring(&config, case);
            assert!(result.is_err(), "Should not match: {}", case);
        }
    }
}
