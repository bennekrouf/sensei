mod call_ollama;
mod first_find_closest_endpoint;
mod models;
mod second_extract_matched_action;
mod third_find_endpoint_by_substring;
mod zero_generate_json;

use crate::first_find_closest_endpoint::find_closest_endpoint;
use crate::zero_generate_json::generate_json;

use models::ConfigFile;
use std::error::Error;

use tracing::{debug, error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting application");

    //example_usage().await?;
    example_usage_with_json().await?;
    Ok(())
}

//pub async fn example_usage() -> Result<(), Box<dyn Error>> {
//    // Initialize tracing
//    info!("Starting example usage tests");
//
//    // Load configuration
//    info!("Loading configuration file");
//    let config_str = tokio::fs::read_to_string("config.yaml").await?;
//    debug!("Config file content length: {}", config_str.len());
//
//    let config: ConfigFile = serde_yaml::from_str(&config_str)?;
//    info!(
//        "Configuration loaded with {} endpoints",
//        config.endpoints.len()
//    );
//
//    // Define test prompts
//    let prompts = vec![
//        //"schedule a meeting tomorrow at 2pm for 1 hour with Salem Mejid to discuss project status",
//        //"send an email to John@gmail.com which title is new report and body is hello john here is the report",
//        //"create a ticket with high priority titled server down and description is production server not responding",
//        //"analyze logs for auth-service from january 1st to today with error level",
//        //"deploy application user-service version 2.1.0 to production with rollback to 2.0.9",
//        //"generate monthly sales report in PDF format",
//        //"backup database users with full backup and high compression",
//        "process payment of 500 USD from customer 12345 using credit card",
//    ];
//
//    // Print test header
//    println!("\n{}", "=".repeat(80));
//    println!("Starting Endpoint Matching Tests");
//    println!("{}\n", "=".repeat(80));
//
//    // Process each prompt
//    for (i, prompt) in prompts.iter().enumerate() {
//        println!("\nTest Case #{}", i + 1);
//        println!("{}", "-".repeat(40));
//        println!("Input: {}", prompt);
//        println!("{}", "-".repeat(40));
//
//        info!("Processing test case #{}: {}", i + 1, prompt);
//
//        match find_closest_endpoint(&config, prompt, "deepseek-r1:8b").await {
//            Ok(result) => {
//                println!("\n✅ Success!");
//                println!("Matched Endpoint ID: {}", result.id);
//                println!("Description: {}", result.description);
//
//                if !result.parameters.is_empty() {
//                    println!("\nRequired Parameters:");
//                    for param in result.parameters.iter().filter(|p| p.required) {
//                        println!("  • {}: {}", param.name, param.description);
//                    }
//
//                    if result.parameters.iter().any(|p| !p.required) {
//                        println!("\nOptional Parameters:");
//                        for param in result.parameters.iter().filter(|p| !p.required) {
//                            println!("  ○ {}: {}", param.name, param.description);
//                        }
//                    }
//                } else {
//                    println!("\nNo parameters required for this endpoint.");
//                }
//
//                info!(
//                    "Successfully matched endpoint '{}' for test case #{}",
//                    result.id,
//                    i + 1
//                );
//            }
//            Err(e) => {
//                println!("\n❌ Error:");
//                println!("Failed to match endpoint: {}", e);
//                error!("Failed to match endpoint for test case #{}: {}", i + 1, e);
//            }
//        }
//
//        println!("\n{}", "=".repeat(80));
//    }
//
//    // Print summary
//    println!("\nTest Summary");
//    println!("{}", "-".repeat(40));
//    println!("Total test cases: {}", prompts.len());
//    println!("\n");
//
//    Ok(())
//}

pub async fn example_usage_with_json() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    info!("Starting example usage tests with JSON generation");

    // Load configuration
    info!("Loading configuration file");
    let config_str = tokio::fs::read_to_string("config.yaml").await?;
    debug!("Config file content length: {}", config_str.len());

    let config: ConfigFile = serde_yaml::from_str(&config_str)?;
    info!(
        "Configuration loaded with {} endpoints",
        config.endpoints.len()
    );

    // Define test prompts
    let prompts = vec![
        //"schedule a meeting tomorrow at 2pm for 1 hour with Salem Mejid to discuss project status",
        //"send an email to John@gmail.com which title is new report and body is hello john here is the report",
        //"create a ticket with high priority titled server down and description is production server not responding",
        //"analyze logs for auth-service from january 1st to today with error level",
        //"deploy application user-service version 2.1.0 to production with rollback to 2.0.9",
        "generate monthly sales report in PDF format action",
        //"backup database users with full backup and high compression",
        //"process payment of 500 USD from customer 12345 using credit card",
    ];

    // Print test header
    println!("\n{}", "=".repeat(80));
    println!("Starting JSON Generation and Endpoint Matching Tests");
    println!("{}\n", "=".repeat(80));

    // Process each prompt
    for (i, prompt) in prompts.iter().enumerate() {
        println!("\nTest Case #{}", i + 1);
        println!("{}", "-".repeat(40));
        println!("Original Input: {}", prompt);
        println!("{}", "-".repeat(40));

        // First, generate JSON
        info!("Generating JSON for test case #{}: {}", i + 1, prompt);
        match generate_json("llama2", prompt).await {
            Ok(json_result) => {
                println!("\n✅ JSON Generation Success!");
                println!("Generated JSON:");
                println!("{}", serde_json::to_string_pretty(&json_result).unwrap());

                // Then proceed with endpoint matching
                info!("Processing endpoint matching for test case #{}", i + 1);
                match find_closest_endpoint(&config, prompt, "deepseek-r1:8b").await {
                    Ok(endpoint_result) => {
                        println!("\n✅ Endpoint Matching Success!");
                        println!("Matched Endpoint ID: {}", endpoint_result.id);
                        println!("Description: {}", endpoint_result.description);

                        // Print parameters
                        if !endpoint_result.parameters.is_empty() {
                            println!("\nRequired Parameters:");
                            for param in endpoint_result.parameters.iter().filter(|p| p.required) {
                                println!("  • {}: {}", param.name, param.description);

                                // Try to find corresponding value in JSON
                                if let Some(endpoints) = json_result.get("endpoints") {
                                    if let Some(first_endpoint) =
                                        endpoints.as_array().and_then(|arr| arr.first())
                                    {
                                        if let Some(fields) = first_endpoint.get("fields") {
                                            if let Some(value) = fields.get(&param.name) {
                                                println!("    ↳ Value from JSON: {}", value);
                                            }
                                        }
                                    }
                                }
                            }

                            if endpoint_result.parameters.iter().any(|p| !p.required) {
                                println!("\nOptional Parameters:");
                                for param in
                                    endpoint_result.parameters.iter().filter(|p| !p.required)
                                {
                                    println!("  ○ {}: {}", param.name, param.description);
                                }
                            }
                        } else {
                            println!("\nNo parameters required for this endpoint.");
                        }

                        info!(
                            "Successfully matched endpoint '{}' for test case #{}",
                            endpoint_result.id,
                            i + 1
                        );
                    }
                    Err(e) => {
                        println!("\n❌ Endpoint Matching Error:");
                        println!("Failed to match endpoint: {}", e);
                        error!("Failed to match endpoint for test case #{}: {}", i + 1, e);
                    }
                }
            }
            Err(e) => {
                println!("\n❌ JSON Generation Error:");
                println!("Failed to generate JSON: {}", e);
                error!("Failed to generate JSON for test case #{}: {}", i + 1, e);
            }
        }

        println!("\n{}", "=".repeat(80));
    }

    // Print summary
    println!("\nTest Summary");
    println!("{}", "-".repeat(40));
    println!("Total test cases: {}", prompts.len());
    println!("\n");

    Ok(())
}
