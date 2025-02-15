use crate::first_find_closest_endpoint::find_closest_endpoint;
use crate::models::ConfigFile;
use crate::zero_sentence_to_json::sentence_to_json;
use std::error::Error;
use tracing::{debug, error, info};

pub async fn example_usage_with_json() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    info!("Starting example usage tests with JSON generation");

    // Load configuration
    info!("Loading configuration file");
    let config_str = tokio::fs::read_to_string("endpoints.yaml").await?;
    debug!("Config file content length: {}", config_str.len());

    let config: ConfigFile = serde_yaml::from_str(&config_str)?;
    info!(
        "Configuration loaded with {} endpoints",
        config.endpoints.len()
    );

    // Define test prompts
    let prompts = vec![
        "schedule a meeting tomorrow at 2pm for 1 hour with Salem Mejid to discuss project status",
        "send an email to John@gmail.com which title is new report and body is hello john here is the report",
        "create a ticket with high priority titled server down and description is production server not responding",
        "analyze logs for auth-service from january 1st to today with error level",
        "deploy application user-service version 2.1.0 to production with rollback to 2.0.9",
        "create monthly sales report in PDF format action",
        "backup database users with full backup and high compression",
        "process payment of 500 USD from customer 12345 using credit card",
    ];

    // Print test header
    info!("\n{}", "=".repeat(80));
    info!("Starting JSON Generation and Endpoint Matching Tests");
    info!("{}\n", "=".repeat(80));

    // Process each prompt
    for (i, prompt) in prompts.iter().enumerate() {
        info!("\nTest Case #{}", i + 1);
        debug!("{}", "-".repeat(40));
        debug!("Original Input: {}", prompt);
        debug!("{}", "-".repeat(40));

        // First, generate JSON
        info!("Generating JSON for test case #{}: {}", i + 1, prompt);
        match sentence_to_json("llama2", prompt).await {
            Ok(json_result) => {
                info!("\n✅ JSON Generation Success!");
                debug!("Generated JSON:");
                debug!("{}", serde_json::to_string_pretty(&json_result).unwrap());

                // Then proceed with endpoint matching
                info!("Processing endpoint matching for test case #{}", i + 1);
                match find_closest_endpoint(&config, prompt, "deepseek-r1:8b").await {
                    Ok(endpoint_result) => {
                        info!("\n✅ Endpoint Matching Success!");
                        info!("Matched Endpoint ID: {}", endpoint_result.id);
                        debug!("Description: {}", endpoint_result.description);

                        // Print parameters
                        if !endpoint_result.parameters.is_empty() {
                            debug!("\nRequired Parameters:");
                            for param in endpoint_result.parameters.iter().filter(|p| p.required) {
                                debug!("  • {}: {}", param.name, param.description);

                                // Try to find corresponding value in JSON
                                if let Some(endpoints) = json_result.get("endpoints") {
                                    if let Some(first_endpoint) =
                                        endpoints.as_array().and_then(|arr| arr.first())
                                    {
                                        if let Some(fields) = first_endpoint.get("fields") {
                                            if let Some(value) = fields.get(&param.name) {
                                                debug!("    ↳ Value from JSON: {}", value);
                                            }
                                        }
                                    }
                                }
                            }

                            if endpoint_result.parameters.iter().any(|p| !p.required) {
                                debug!("\nOptional Parameters:");
                                for param in
                                    endpoint_result.parameters.iter().filter(|p| !p.required)
                                {
                                    debug!("  ○ {}: {}", param.name, param.description);
                                }
                            }
                        } else {
                            debug!("\nNo parameters required for this endpoint.");
                        }

                        info!(
                            "Successfully matched endpoint '{}' for test case #{}",
                            endpoint_result.id,
                            i + 1
                        );
                    }
                    Err(e) => {
                        error!("\n❌ Endpoint Matching Error:");
                        error!("Failed to match endpoint: {}", e);
                        error!("Failed to match endpoint for test case #{}: {}", i + 1, e);
                    }
                }
            }
            Err(e) => {
                error!("\n❌ JSON Generation Error:");
                error!("Failed to generate JSON: {}", e);
                error!("Failed to generate JSON for test case #{}: {}", i + 1, e);
            }
        }

        info!("\n{}", "=".repeat(80));
    }

    // Print summary
    info!("\nTest Summary");
    debug!("{}", "-".repeat(40));
    info!("Total test cases: {}", prompts.len());
    info!("\n");

    Ok(())
}
