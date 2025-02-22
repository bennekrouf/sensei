This file is a merged representation of the entire codebase, combined into a single document by Repomix.

# File Summary

## Purpose
This file contains a packed representation of the entire repository's contents.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.

## File Format
The content is organized as follows:
1. This summary section
2. Repository information
3. Directory structure
4. Multiple file entries, each consisting of:
  a. A header with the file path (## File: path/to/file)
  b. The full contents of the file in a code block

## Usage Guidelines
- This file should be treated as read-only. Any changes should be made to the
  original repository files, not this packed version.
- When processing this file, use the file path to distinguish
  between different files in the repository.
- Be aware that this file may contain sensitive information. Handle it with
  the same level of security as you would the original repository.

## Notes
- Some files may have been excluded based on .gitignore rules and Repomix's configuration
- Binary files are not included in this packed representation. Please refer to the Repository Structure section for a complete list of file paths, including binary files
- Files matching patterns in .gitignore are excluded
- Files matching default ignore patterns are excluded

## Additional Info

# Directory Structure
```
src/
  models/
    config.rs
    mod.rs
  prompts/
    mod.rs
  analyze_sentence.rs
  call_ollama.rs
  cli.rs
  config.yaml
  example_usage_with_json.rs
  first_find_closest_endpoint.rs
  fourth_match_fields.rs
  grpc_server.rs
  json_helper.rs
  main.rs
  second_extract_matched_action.rs
  sentence_service.rs
  third_find_endpoint_by_substring.rs
  zero_sentence_to_json.rs
test/
  sentence.sh
.gitignore
build.rs
Cargo.toml
cert.pem
config.yaml
endpoints.yaml
prompts.yaml
README.md
sentence_service.proto
```

# Files

## File: src/models/config.rs
````rust
use serde::Deserialize;
use std::error::Error;
use tracing::debug;

#[derive(Debug, Deserialize, Clone)]
pub struct ModelConfig {
    pub name: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelsConfig {
    pub sentence_to_json: ModelConfig,
    pub find_endpoint: ModelConfig,
    pub semantic_match: ModelConfig,
}

pub async fn load_models_config() -> Result<ModelsConfig, Box<dyn Error + Send + Sync>> {
    let config_str = tokio::fs::read_to_string("config.yaml").await?;
    let config: serde_yaml::Value = serde_yaml::from_str(&config_str)?;

    let models_config = config["models"]
        .as_mapping()
        .ok_or("No models configuration found")?;

    let models: ModelsConfig = serde_yaml::from_value(config["models"].clone())?;

    debug!("Loaded models configuration: {:#?}", models);

    Ok(models)
}
````

## File: src/models/mod.rs
````rust
pub mod config;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub format: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct OllamaResponse {
    pub response: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Endpoint {
    pub id: String,
    pub text: String,
    pub description: String,
    pub parameters: Vec<EndpointParameter>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EndpointParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub alternatives: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    pub endpoints: Vec<Endpoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub description: String,
    pub semantic_value: Option<String>,
}
````

## File: src/prompts/mod.rs
````rust
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct PromptTemplate {
    template: String,
}

#[derive(Debug, Deserialize)]
pub struct PromptConfig {
    prompts: std::collections::HashMap<String, PromptTemplate>,
}

pub struct PromptManager {
    config: PromptConfig,
}

impl PromptManager {
    pub async fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let config_str = tokio::fs::read_to_string("prompts.yaml").await?;
        let config: PromptConfig = serde_yaml::from_str(&config_str)?;
        Ok(Self { config })
    }

    pub fn get_prompt(&self, name: &str) -> Option<&str> {
        self.config.prompts.get(name).map(|t| t.template.as_str())
    }

    pub fn format_find_endpoint(&self, input_sentence: &str, actions_list: &str) -> String {
        let template = self.get_prompt("find_endpoint").unwrap_or_default();
        template
            .replace("{input_sentence}", input_sentence)
            .replace("{actions_list}", actions_list)
    }

    pub fn format_sentence_to_json(&self, sentence: &str) -> String {
        let template = self.get_prompt("sentence_to_json").unwrap_or_default();
        template.replace("{sentence}", sentence)
    }
}
````

## File: src/analyze_sentence.rs
````rust
use crate::first_find_closest_endpoint::find_closest_endpoint;
use crate::fourth_match_fields::match_fields_semantic;
use crate::models::config::load_models_config;
use crate::models::ConfigFile;
use crate::models::Parameter as ServiceParameter;
use crate::zero_sentence_to_json::sentence_to_json;
use serde_json::Value;
use std::error::Error;
use tracing::{debug, info};

pub struct AnalysisResult {
    pub json_output: Value,
    pub endpoint_id: String,
    pub endpoint_description: String,
    pub parameters: Vec<ServiceParameter>,
}

pub async fn analyze_sentence(
    sentence: &str,
) -> Result<AnalysisResult, Box<dyn Error + Send + Sync>> {
    info!("Starting sentence analysis for: {}", sentence);

    // Load configurations
    let config_str = tokio::fs::read_to_string("endpoints.yaml").await?;
    debug!("Config file content length: {}", config_str.len());

    let config: ConfigFile = serde_yaml::from_str(&config_str)?;
    info!(
        "Configuration loaded with {} endpoints",
        config.endpoints.len()
    );

    // Load model configurations
    let models_config = load_models_config().await?;
    debug!("Model configurations loaded successfully");

    // Generate JSON from sentence
    info!("Generating JSON from sentence");
    let json_result = sentence_to_json(sentence).await?;
    debug!("JSON generation successful");

    // Find closest matching endpoint
    info!("Finding closest matching endpoint");
    let endpoint_result = find_closest_endpoint(&config, sentence).await?;
    debug!("Endpoint matching successful: {}", endpoint_result.id);

    // Perform both exact and semantic matching
    let mut parameters = Vec::new();

    // Exact matching
    for param in &endpoint_result.parameters {
        let mut exact_value = None;
        if let Some(endpoints) = json_result.get("endpoints") {
            if let Some(first_endpoint) = endpoints.as_array().and_then(|arr| arr.first()) {
                if let Some(fields) = first_endpoint.get("fields") {
                    exact_value = fields.get(&param.name).map(|v| v.to_string());
                }
            }
        }

        // Semantic matching
        let semantic_results = match_fields_semantic(&json_result, &endpoint_result).await?;
        let semantic_value = semantic_results
            .iter()
            .find(|(name, _, _)| name == &param.name)
            .and_then(|(_, _, value)| value.clone());

        parameters.push(ServiceParameter {
            name: param.name.clone(),
            description: param.description.clone(),
            semantic_value,
        });
    }

    debug!("Analysis completed successfully");
    Ok(AnalysisResult {
        json_output: json_result,
        endpoint_id: endpoint_result.id,
        endpoint_description: endpoint_result.description,
        parameters,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyze_sentence() {
        let test_sentence = "schedule a meeting tomorrow at 2pm with John";
        match analyze_sentence(test_sentence).await {
            Ok(result) => {
                assert!(!result.endpoint_id.is_empty());
                assert!(!result.endpoint_description.is_empty());
                // Add more assertions as needed
            }
            Err(e) => panic!("Analysis failed: {}", e),
        }
    }
}
````

## File: src/call_ollama.rs
````rust
use crate::models::config::ModelConfig;
use crate::models::GenerateRequest;
use crate::models::OllamaResponse;

use std::error::Error;
use tracing::{debug, error, info};

pub async fn call_ollama_with_config(
    model_config: &ModelConfig,
    prompt: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    debug!("Creating Ollama request for model: {}", model_config.name);
    let client = reqwest::Client::new();
    let request_body = GenerateRequest {
        model: model_config.name.clone(),
        prompt: prompt.to_string(),
        stream: false,
        format: None,
        temperature: model_config.temperature,
        max_tokens: model_config.max_tokens,
    };

    info!("Sending request to Ollama");
    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request_body)
        .send()
        .await?;

    debug!("Response status: {}", response.status());

    if !response.status().is_success() {
        error!("Ollama request failed with status: {}", response.status());
        return Err(format!("Request failed with status: {}", response.status()).into());
    }

    let response_obj = response.json::<OllamaResponse>().await?;

    if response_obj.response.trim().is_empty() {
        error!("Received empty response from Ollama");
        return Err("Empty response from Ollama".into());
    }

    debug!("Parsed response: '{}'", response_obj.response.trim());

    Ok(response_obj.response.trim().to_owned())
}
````

## File: src/cli.rs
````rust
// src/cli.rs
use crate::analyze_sentence::analyze_sentence;
use clap::Parser;
use std::error::Error;
use tracing::info;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The sentence to analyze (if not provided, starts gRPC server)
    pub prompt: Option<String>,
}

pub async fn handle_cli(cli: Cli) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(prompt) = cli.prompt {
        info!("Analyzing prompt via CLI: {}", prompt);
        let result = analyze_sentence(&prompt).await?;

        println!("\nAnalysis Results:");
        println!(
            "Endpoint: {} ({})",
            result.endpoint_id, result.endpoint_description
        );
        println!("\nParameters:");
        for param in result.parameters {
            println!("\n{} ({}):", param.name, param.description);
            if let Some(semantic) = param.semantic_value {
                println!("  Semantic Match: {}", semantic);
            }
        }

        println!("\nRaw JSON Output:");
        println!("{}", serde_json::to_string_pretty(&result.json_output)?);
    }
    Ok(())
}
````

## File: src/config.yaml
````yaml
endpoints:
  - id: "send_email"
    text: "send email"
    description: "Send an email with possible attachments"
    parameters:
      - name: "to"
        description: "Recipient's email address"
        required: true
        alternatives:
          - "recipient_email"
          - "email_to"
          - "to_email"
          - "destination_email"
      - name: "subject"
        description: "Email subject"
        required: true
        alternatives:
          - "email_title"
          - "mail_subject"
          - "title"
          - "email_subject"
      - name: "body"
        description: "Email content"
        required: true
        alternatives:
          - "email_body"
          - "content"
          - "message"
          - "mail_content"
          - "email_content"
      - name: "attachments"
        description: "Attachments"
        required: false
        alternatives:
          - "files"
          - "attached_files"
          - "email_attachments"

  - id: "create_ticket"
    text: "Create a new support ticket for tracking and resolving customer issues"
    description: "Create a new support ticket for tracking and resolving customer issues"
    parameters:
      - name: "title"
        description: "Ticket title"
        required: true
        alternatives:
          - "ticket_title"
          - "issue_title"
          - "ticket_name"
          - "issue_name"
      - name: "priority"
        description: "Ticket priority (urgent, normal, low)"
        required: true
        alternatives:
          - "ticket_priority"
          - "urgency"
          - "importance"
          - "severity"
      - name: "description"
        description: "Detailed problem description"
        required: true
        alternatives:
          - "ticket_description"
          - "issue_description"
          - "problem_details"
          - "details"
          - "issue_content"

  - id: "schedule_meeting"
    text: "schedule meeting"
    description: "Schedule a meeting or appointment"
    parameters:
      - name: "date"
        description: "Meeting date"
        required: true
        alternatives:
          - "meeting_date"
          - "appointment_date"
          - "scheduled_date"
          - "event_date"
      - name: "time"
        description: "Meeting time"
        required: true
        alternatives:
          - "meeting_time"
          - "appointment_time"
          - "scheduled_time"
          - "start_time"
          - "event_time"
      - name: "participants"
        description: "List of participants"
        required: true
        alternatives:
          - "attendees"
          - "meeting_participants"
          - "invitees"
          - "members"
          - "people"
      - name: "duration"
        description: "Duration in minutes"
        required: true
        alternatives:
          - "meeting_duration"
          - "length"
          - "time_duration"
          - "duration_minutes"
      - name: "topic"
        description: "Meeting topic"
        required: false
        alternatives:
          - "meeting_topic"
          - "subject"
          - "agenda"
          - "meeting_subject"

  - id: "analyze_logs"
    text: "analyze logs"
    description: "Analyze application logs"
    parameters:
      - name: "app_name"
        description: "Application name"
        required: true
        alternatives:
          - "application_name"
          - "app"
          - "application"
          - "service_name"
      - name: "start_date"
        description: "Analysis start date"
        required: true
        alternatives:
          - "from_date"
          - "begin_date"
          - "analysis_start"
          - "log_start_date"
      - name: "end_date"
        description: "Analysis end date"
        required: true
        alternatives:
          - "to_date"
          - "finish_date"
          - "analysis_end"
          - "log_end_date"
      - name: "log_level"
        description: "Log level (ERROR, WARN, INFO, DEBUG)"
        required: false
        alternatives:
          - "level"
          - "severity_level"
          - "logging_level"
          - "debug_level"

  - id: "deploy_app"
    text: "deploy application"
    description: "Deploy an application to production"
    parameters:
      - name: "app_name"
        description: "Application name to deploy"
        required: true
        alternatives:
          - "application_name"
          - "app"
          - "service_name"
          - "deployment_name"
      - name: "version"
        description: "Version to deploy"
        required: true
        alternatives:
          - "app_version"
          - "release_version"
          - "deployment_version"
          - "build_version"
      - name: "environment"
        description: "Target environment (prod, staging, dev)"
        required: true
        alternatives:
          - "env"
          - "target_env"
          - "deployment_env"
          - "target_environment"
      - name: "rollback_version"
        description: "Rollback version in case of error"
        required: false
        alternatives:
          - "backup_version"
          - "fallback_version"
          - "previous_version"
          - "revert_version"

  - id: "generate_report"
    text: "generate report"
    description: "Generate analysis or statistics report"
    parameters:
      - name: "report_type"
        description: "Report type (sales, traffic, performance)"
        required: true
        alternatives:
          - "type"
          - "kind"
          - "report_kind"
          - "analysis_type"
      - name: "period"
        description: "Report period (daily, weekly, monthly)"
        required: true
        alternatives:
          - "time_period"
          - "duration"
          - "report_period"
          - "time_range"
      - name: "format"
        description: "Output format (PDF, Excel, CSV)"
        required: true
        alternatives:
          - "output_format"
          - "file_format"
          - "report_format"
          - "export_format"

  - id: "backup_database"
    text: "backup database"
    description: "Create a database backup"
    parameters:
      - name: "database"
        description: "Database name"
        required: true
        alternatives:
          - "db_name"
          - "db"
          - "database_name"
          - "schema_name"
      - name: "backup_type"
        description: "Backup type (full, incremental)"
        required: true
        alternatives:
          - "type"
          - "backup_mode"
          - "db_backup_type"
          - "backup_method"
      - name: "compression"
        description: "Compression level (none, low, high)"
        required: false
        alternatives:
          - "compression_level"
          - "compress_level"
          - "compress_type"
          - "compression_type"

  - id: "process_payment"
    text: "process payment means paying something about business and paiement not send email"
    description: "Process a customer payment"
    parameters:
      - name: "amount"
        description: "Payment amount"
        required: true
        alternatives:
          - "payment_amount"
          - "sum"
          - "total"
          - "price"
      - name: "currency"
        description: "Currency (EUR, USD)"
        required: true
        alternatives:
          - "currency_code"
          - "currency_type"
          - "payment_currency"
          - "money_type"
      - name: "payment_method"
        description: "Payment method (card, transfer, paypal)"
        required: true
        alternatives:
          - "method"
          - "pay_method"
          - "payment_type"
          - "pay_type"
      - name: "customer_id"
        description: "Customer identifier"
        required: true
        alternatives:
          - "client_id"
          - "user_id"
          - "payer_id"
          - "customer_number"
````

## File: src/example_usage_with_json.rs
````rust
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
````

## File: src/first_find_closest_endpoint.rs
````rust
use crate::models::config::load_models_config;
use crate::models::ConfigFile;
use crate::models::Endpoint;
use crate::prompts::PromptManager;
use crate::second_extract_matched_action::extract_matched_action;
use crate::{
    call_ollama::call_ollama_with_config,
    third_find_endpoint_by_substring::find_endpoint_by_substring,
};
use std::error::Error;
use tracing::{debug, error, info};

pub async fn find_closest_endpoint(
    config: &ConfigFile,
    input_sentence: &str,
) -> Result<Endpoint, Box<dyn Error + Send + Sync>> {
    info!("Starting endpoint matching for input: {}", input_sentence);
    debug!("Available endpoints: {}", config.endpoints.len());

    // Load model configuration
    let models_config = load_models_config().await?;
    let model_config = &models_config.find_endpoint;

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

    // Call Ollama with configuration
    info!("Calling Ollama with model: {}", model_config.name);
    let raw_response = call_ollama_with_config(model_config, &prompt).await?;
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
````

## File: src/fourth_match_fields.rs
````rust
use crate::models::config::load_models_config;
use crate::models::Endpoint;
use crate::{call_ollama::call_ollama_with_config, json_helper::sanitize_json};
use serde_json::Value;
use std::error::Error;
use tracing::{debug, info};

pub async fn match_fields_semantic(
    input_json: &Value,
    endpoint: &Endpoint,
) -> Result<Vec<(String, String, Option<String>)>, Box<dyn Error + Send + Sync>> {
    let input_fields = if let Some(endpoints) = input_json.get("endpoints") {
        if let Some(first_endpoint) = endpoints.as_array().and_then(|arr| arr.first()) {
            if let Some(fields) = first_endpoint.get("fields") {
                if let Some(obj) = fields.as_object() {
                    obj.iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect::<Vec<_>>()
                        .join(", ")
                } else {
                    return Err("Fields is not an object".into());
                }
            } else {
                return Err("No fields found in JSON".into());
            }
        } else {
            return Err("No endpoints found in JSON".into());
        }
    } else {
        return Err("Invalid JSON structure".into());
    };

    let parameters = endpoint
        .parameters
        .iter()
        .map(|p| {
            format!(
                "{}: {} (alternatives: {})",
                p.name,
                p.description,
                p.alternatives.join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "Given these input fields from a sentence: '{input_fields}'\n\
        And these endpoint parameters:\n{parameters}\n\n\
        For each endpoint parameter:\n\
        1. Look at the input fields\n\
        2. Find any field that matches the parameter or its alternatives\n\
        3. Extract the actual value from the matching input field\n\n\
        Return a JSON where:\n\
        - Keys are the endpoint parameter names\n\
        - Values are the actual values found in the input fields\n\
        Only include parameters where you found a matching value.\n\
        Return valid JSON only, no additional text.",
    );

    debug!("Field matching prompt:\n{}", prompt);
    info!("Calling Ollama for field matching");

    // Load model configuration
    let models_config = load_models_config().await?;
    let model_config = &models_config.sentence_to_json;

    let response = call_ollama_with_config(model_config, &prompt).await?;
    let json_response = sanitize_json(&response)?;

    debug!("Semantic matching response: {:?}", json_response);

    let mut matched_fields = Vec::new();
    let input_fields = input_json["endpoints"][0]["fields"]
        .as_object()
        .ok_or("Invalid JSON structure")?;

    for param in &endpoint.parameters {
        // First try exact match
        let mut value = input_fields.get(&param.name).map(|v| v.to_string());

        // If no exact match, try alternatives
        if value.is_none() {
            for alt in &param.alternatives {
                if let Some(v) = input_fields.get(alt) {
                    value = Some(v.to_string());
                    break;
                }
            }
        }

        // If still no match, check semantic matching result
        if value.is_none() {
            if let Some(v) = json_response.get(&param.name) {
                value = Some(v.to_string().trim_matches('"').to_string());
            }
        }

        matched_fields.push((param.name.clone(), param.description.clone(), value));
    }

    Ok(matched_fields)
}
````

## File: src/grpc_server.rs
````rust
use tonic_reflection::server::Builder;
use tracing::info;
use crate::sentence_service::SentenceAnalyzeService;
use crate::sentence_service::sentence::sentence_service_server::SentenceServiceServer;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{Any, CorsLayer};

pub async fn start_sentence_grpc_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = "0.0.0.0:50053".parse()?;
    info!("Starting sentence analysis gRPC server on {}", addr);

    let descriptor_set = include_bytes!(concat!(env!("OUT_DIR"), "/sentence_descriptor.bin"));
    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(descriptor_set)
        .build_v1()?;

    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any)
        .expose_headers(Any);

    tracing::info!("Starting semantic gRPC server on {}", addr);

    let sentence_service = SentenceAnalyzeService::default();
    let service = SentenceServiceServer::new(sentence_service);

    match Server::builder()
        .accept_http1(true)
        .max_concurrent_streams(128) // Set reasonable limits
        .tcp_keepalive(Some(std::time::Duration::from_secs(60)))
        .tcp_nodelay(true)
        .layer(cors) // Add CORS layer
        .layer(GrpcWebLayer::new())
        .add_service(service)
        .add_service(reflection_service) // Add reflection service
        .serve_with_shutdown(addr, async {
            tokio::signal::ctrl_c().await.ok();
            info!("Shutting down semantic server...");
        })
        .await
    {
    Ok(_) => Ok::<(), Box<dyn std::error::Error + Send + Sync>>(()),
        Err(e) => {
            if e.to_string().contains("Address already in use") {
                tracing::error!("Port already in use. Please stop other instances first.");
            }
            Err(e.into())
        }
    }
}
````

## File: src/json_helper.rs
````rust
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
````

## File: src/main.rs
````rust
mod analyze_sentence;
mod call_ollama;
mod cli;
//mod example_usage_with_json;
mod first_find_closest_endpoint;
mod fourth_match_fields;
mod grpc_server;
mod json_helper;
mod models;
mod prompts;
mod second_extract_matched_action;
mod sentence_service;
mod third_find_endpoint_by_substring;
mod zero_sentence_to_json;

// use example_usage_with_json::example_usage_with_json;
use clap::Parser;
use cli::{handle_cli, Cli};
use grpc_logger::load_config;
use grpc_logger::setup_logging;
use grpc_server::start_sentence_grpc_server;
use std::error::Error;
use tokio::signal;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let config = load_config("config.yaml")?;
    setup_logging(&config).await?;

    // Parse CLI arguments
    let cli = Cli::parse();

    // Start the gRPC server
    //let grpc_server = tokio::spawn(async {
    //    if let Err(e) = start_sentence_grpc_server().await {
    //        error!("gRPC server error: {:?}", e);
    //    }
    //});

    //info!("Semantic server started");

    // Example task (if needed)
    // let example_task = tokio::spawn(async {
    //     if let Err(e) = example_usage_with_json().await {
    //         info!("Error in semantic application: {:?}", e);
    //     }
    // });

    // Wait for either CTRL-C or task completion
    //tokio::select! {
    //    _ = signal::ctrl_c() => {
    //        info!("Received shutdown signal, initiating graceful shutdown...");
    //    }
    //    result = grpc_server => {
    //        if let Err(e) = result {
    //            error!("gRPC server task error: {:?}", e);
    //        }
    //    }
    // result = example_task => {
    //     if let Err(e) = result {
    //         info!("Example task error: {:?}", e);
    //     }
    //    // }
    //}

    //info!("Server shutting down");

    // Handle CLI command if present, otherwise start gRPC server
    match cli.prompt {
        Some(_) => {
            handle_cli(cli).await?;
        }
        None => {
            info!("No prompt provided, starting gRPC server...");

            // Start the gRPC server
            let grpc_server = tokio::spawn(async {
                if let Err(e) = start_sentence_grpc_server().await {
                    error!("gRPC server error: {:?}", e);
                }
            });

            info!("Semantic server started");

            // Wait for CTRL-C
            tokio::select! {
                _ = signal::ctrl_c() => {
                    info!("Received shutdown signal, initiating graceful shutdown...");
                }
                result = grpc_server => {
                    if let Err(e) = result {
                        error!("gRPC server task error: {:?}", e);
                    }
                }
            }

            info!("Server shutting down");
        }
    }

    Ok(())
}
````

## File: src/second_extract_matched_action.rs
````rust
use std::error::Error;

use tracing::{debug, error};

pub async fn extract_matched_action(ollama_response: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
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
````

## File: src/sentence_service.rs
````rust
use crate::analyze_sentence::analyze_sentence;
use futures::Stream;
use std::pin::Pin;
use tokio::sync::mpsc;
use tonic::metadata::MetadataMap;
use tonic::{Request, Response, Status};

pub mod sentence {
    tonic::include_proto!("sentence");
}

use sentence::sentence_service_server::SentenceService;
use sentence::{Parameter, SentenceRequest, SentenceResponse};
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tracing::Instrument;

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
    type AnalyzeSentenceStream =
        Pin<Box<dyn Stream<Item = Result<SentenceResponse, Status>> + Send>>;

    #[tracing::instrument(skip(self, request), fields(client_id))]
    async fn analyze_sentence(
        &self,
        request: Request<SentenceRequest>,
    ) -> Result<Response<Self::AnalyzeSentenceStream>, Status> {
        let metadata = request.metadata().clone();
        // Log request details
        tracing::info!("Request metadata: {:?}", metadata);
        tracing::info!("Request headers: {:?}", metadata.keys());

        let client_id = Self::get_client_id(&metadata);
        let sentence = request.into_inner().sentence;
        tracing::info!(sentence = %sentence, "Processing sentence request");

        // Debug logging for request details
        tracing::debug!(
            "Full request details: {:?}",
            metadata
                .iter()
                .map(|item| match item {
                    tonic::metadata::KeyAndValueRef::Ascii(k, v) =>
                        (k.as_str(), v.to_str().unwrap_or("invalid")),
                    tonic::metadata::KeyAndValueRef::Binary(k, _) => (k.as_str(), "binary value"),
                })
                .collect::<Vec<_>>()
        );

        let (tx, rx) = mpsc::channel(10);
        let analyze_span = tracing::info_span!("analyze_sentence", client_id = %client_id);

        tokio::spawn(async move {
            let result = analyze_sentence(&sentence).instrument(analyze_span).await;

            match result {
                Ok(result) => {
                    tracing::info!(client_id = %client_id, "Analysis completed");

                    let response = SentenceResponse {
                        endpoint_id: result.endpoint_id,
                        endpoint_description: result.endpoint_description,
                        parameters: result
                            .parameters
                            .into_iter()
                            .map(|param| Parameter {
                                name: param.name,
                                description: param.description,
                                semantic_value: param.semantic_value,
                            })
                            .collect(),
                        json_output: match serde_json::to_string(&result.json_output) {
                            Ok(json) => json,
                            Err(e) => {
                                tracing::error!(error = %e, "JSON serialization failed");
                                format!("{{\"error\": \"JSON serialization failed: {}\"}}", e)
                            }
                        },
                    };

                    tracing::info!(
                        client_id = %client_id,
                        response = ?response,
                        "Sending response"
                    );

                    if tx.send(Ok(response)).await.is_err() {
                        tracing::error!(client_id = %client_id, "Failed to send response - stream closed");
                    }
                }
                Err(e) => {
                    tracing::error!(
                        sentence = %sentence,
                        error = %e,
                        client_id = %client_id,
                        "Analysis failed"
                    );

                    let _ = tx
                        .send(Err(Status::internal(format!("Analysis failed: {}", e))))
                        .await;
                }
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }
}
````

## File: src/third_find_endpoint_by_substring.rs
````rust
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
````

## File: src/zero_sentence_to_json.rs
````rust
use crate::json_helper::sanitize_json;
use crate::models::config::load_models_config;
use crate::{call_ollama::call_ollama_with_config, prompts::PromptManager};
use std::error::Error;
use tracing::{debug, error, info};

pub async fn sentence_to_json(
    sentence: &str,
) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
    let prompt_manager = PromptManager::new().await?;
    let full_prompt = prompt_manager.format_sentence_to_json(sentence);

    // Load model configuration
    let models_config = load_models_config().await?;
    let model_config = &models_config.sentence_to_json;

    let full_response_text = call_ollama_with_config(model_config, &full_prompt).await?;
    debug!("Raw LLM response:\n{}", full_response_text);

    let parsed_json = sanitize_json(&full_response_text)?;

    // Validate the JSON structure - Fixed the condition
    if !parsed_json.is_object() || !parsed_json.get("endpoints").is_some() {
        error!("Invalid JSON structure: missing 'endpoints' array");
        return Err("Invalid JSON structure: missing 'endpoints' array".into());
    }

    // Additional validation to ensure endpoints is an array and has at least one item
    let endpoints = parsed_json
        .get("endpoints")
        .and_then(|e| e.as_array())
        .ok_or_else(|| {
            error!("Invalid JSON structure: 'endpoints' is not an array");
            "Invalid JSON structure: 'endpoints' is not an array"
        })?;

    if endpoints.is_empty() {
        error!("Invalid JSON structure: 'endpoints' array is empty");
        return Err("Invalid JSON structure: 'endpoints' array is empty".into());
    }

    info!("Successfully generated and validated JSON");
    Ok(parsed_json)
}
````

## File: test/sentence.sh
````bash
#!/bin/bash

# Configuration
HOST="0.0.0.0:50053" # Match your gRPC server address

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to test streaming endpoint
test_streaming_endpoint() {
  local sentence="$1"
  local description="$2"

  echo -e "${BLUE}Testing: $description${NC}"
  echo "Sentence: $sentence"
  echo "-----------------"

  REQUEST_PAYLOAD=$(
    cat <<EOF
{
    "sentence": "$sentence"
}
EOF
  )

  echo "Request payload:"
  echo "$REQUEST_PAYLOAD"
  echo "-----------------"

  # Call gRPC streaming endpoint
  response=$(grpcurl -plaintext \
    -d "$REQUEST_PAYLOAD" \
    $HOST \
    sentence.SentenceService/AnalyzeSentence 2>&1)

  if [ $? -eq 0 ]; then
    echo -e "${GREEN}Success:${NC}"
    echo "$response"
  else
    echo -e "${RED}Error:${NC}"
    echo "$response"
  fi
  echo "-----------------"
  echo
}

# Test streaming response
test_streaming_endpoint "Analyze this sentence" "Streaming test"

# List available services (for verification)
echo "Checking available services:"
echo "-----------------"
grpcurl -plaintext $HOST list
echo

# Show service description
echo "Service description:"
echo "-----------------"
grpcurl -plaintext $HOST describe sentence.SentenceService
echo

echo "All tests completed."
````

## File: .gitignore
````
/target
logs
````

## File: build.rs
````rust
use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // Compile sentence service proto
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("sentence_descriptor.bin"))
        .compile_protos(&["sentence_service.proto"], &["proto"])
        .unwrap_or_else(|e| panic!("Failed to compile proto files: {}", e));
}
````

## File: Cargo.toml
````toml
[package]
name = "apicheck"
version = "0.1.0"
edition = "2021"

[dependencies]
# grpc_logger = "0.10.0"
grpc_logger = { path = "../grpc-logger" } 
regex = "1.11.1"
reqwest = { version = "0.12.12", features = ["json"] }
serde = { version = "1.0.217", features = ["derive"] }
serde_json = "1.0.138"
serde_yaml = "0.9.34"
tokio = { version = "1.43.0", features = ["full"] }
tracing = "0.1.41"
tracing-subscriber = "0.3.19"
tonic = { version = "0.12.3", features = ["gzip", "tls"] }
tonic-reflection = "0.12.3"
tonic-web = "0.12.3"
tower-http = { version = "0.6.2", features = ["cors"] }
prost = "0.13.4"
tracing-futures = "0.2.5"
http = "1.2.0"
tokio-stream = "0.1.17"
futures = "0.3.31"
clap = { version = "4.5.30", features = ["derive"] }

[build-dependencies]
tonic-build = "0.12.3"
````

## File: cert.pem
````
-----BEGIN CERTIFICATE-----
MIIFazCCA1OgAwIBAgIUecTLkq9dX/A4P5Cne8aocCGRF1MwDQYJKoZIhvcNAQEL
BQAwRTELMAkGA1UEBhMCQVUxEzARBgNVBAgMClNvbWUtU3RhdGUxITAfBgNVBAoM
GEludGVybmV0IFdpZGdpdHMgUHR5IEx0ZDAeFw0yNTAyMTIxMzMxNDJaFw0yNjAy
MTIxMzMxNDJaMEUxCzAJBgNVBAYTAkFVMRMwEQYDVQQIDApTb21lLVN0YXRlMSEw
HwYDVQQKDBhJbnRlcm5ldCBXaWRnaXRzIFB0eSBMdGQwggIiMA0GCSqGSIb3DQEB
AQUAA4ICDwAwggIKAoICAQClcu57qt2QVnA/msrXXjof80JfCfReUryfo5etUWhj
2lj3O1f2oakfDjDXzWN6OZdQVFxD1u7Q/xH4gPbzbVfHarEFsF2CQ7RmChXsebMm
7DMiRNzc27gMlWhNUClfjyOWlxHtrYFn1bVZmVPbVnB/AgywtnshuPkZqLF8rukm
hUYPg0OWrad0Mfd19dhLIkZfqfTEU4byCstNjek8CHM5bMC7g/XG1ObUAg83ib6V
W19ttrG0m5JStorOX2l45gqj9xWMmXoOXncbUH+XMRgxLYS467+i4Q5k3b3JUsgJ
CGnwnf/7jx4zxeXYe1HvVauGNdAPFnQwmRglcJ/Vb1NI9ap7dftrkxPo4DSQsHiK
MhbrHTd1f4UpvXu6xSI6f7z19vfVBAE5jHbnBZ8IJPdJGS+EF/nZCtt6C7t+BoMu
o2uMpVB0RVctxi6zsDNUW4+Vor13GMy4f+CW/6RcxQ1jCmzPD3IgF2nLPF+XKeu9
7uWQptFnW1VyYD6KS0mUxFQUBzh0iJBkha3agNFBSwetJ0rINQxKR4DFVtf0XEUD
prlDg3e16Ka7YF8giwuxsS5u0Yw0h50DMaXeNF/TvC+hVHBLoZW5+EsOHNNSLcc/
PViMWofwOen6y19bg+VLH+hAc3zZLYfuXbBnUB8dsRs/Ter1w51zFVGWbgBS7/sZ
iwIDAQABo1MwUTAdBgNVHQ4EFgQU9pA/VBwqGit5f67+BjJR1A/creYwHwYDVR0j
BBgwFoAU9pA/VBwqGit5f67+BjJR1A/creYwDwYDVR0TAQH/BAUwAwEB/zANBgkq
hkiG9w0BAQsFAAOCAgEAmdQP/cK5PysXsXnwHLx/tQ1tdGsxPZtJTttFvFh2CBhw
z3ydjc7GB23w4nPY5Tdvd7wniXs+Zv+eNvO1EwGaHb7kVTns9j2iCVg/TePzGvZa
n7eIgug+oDnBDGCV64D3rWnF/bKrUc+wzuw5OBBKbgHBZSzoUO3XldkjRtsxRvKy
LJFd4Ha9rgdZX3MNpuheCT+FQOXamHH6NZcVdjzK0aOfnp51kZvqUTgaf1Ud9TX0
wM0ZweOjz6HSu9eh89mZu5XgkD0sHX8g5VnEt+znvkX2y1MWyhdWPFDdMXLBLbft
rxdF7fKKEBRLL+1fkVGL9ySUbCEddIDJvpc8eULJ7/RPYhcnye++VdwSR3FsoXdB
pqjB/7L/qCE0po7TpPnY3qiSS63bCM3ogYodyh17/YKNPZNj2/axg0VMbmHMisKx
DROmY1e8xCM1Y6sj5x6OjvZklrIKira03M1cIZlcjDk8UZbMiNUb00x/qDebgIG6
yh4dFovy/ZA/8ereyjNYqtdT2bQroLevRUQRiLyUVRIviB53m+BQN2B0MpUdtxIy
k0/v/rU+IDteSLXPXuk3X+faQRMfgazlNfaf3IQJZpFZsxm6iM8vCzad+ur47X8u
Vvf/ShSCoUBH2H0VNoOU3SIH8g5yVOuFKeaZUpBDXg78OV4z4SjN07D+C2A6QVU=
-----END CERTIFICATE-----
````

## File: config.yaml
````yaml
output: grpc
level: info
server_id: "semantic-service"  # Identifies this service in the logs
client_id: "semantic-service"
grpc:
  address: "127.0.0.1"
  port: 50052 # Port where your grpc-logger server is running
log_fields:
  include_thread_id: true
  include_target: true
  include_file: true
  include_line: true
  include_timestamp: true
client_retry:
  max_retries: 5000
  base_delay_secs: 1
  reconnect_delay_secs: 5
log_all_messages: false

debug_mode:
  enabled: false
  test_interval_secs: 10

models:
  sentence_to_json:
    name: "llama2"
    temperature: 0.1
    max_tokens: 1000
  find_endpoint:
    name: "deepseek-r1:8b"
    temperature: 0.1
    max_tokens: 500
  semantic_match:
    name: "deepseek-r1:8b"
    temperature: 0.1
    max_tokens: 500
````

## File: endpoints.yaml
````yaml
endpoints:
  - id: "send_email"
    text: "send email"
    description: "Send an email with possible attachments"
    parameters:
      - name: "to"
        description: "Recipient's email address"
        required: true
        alternatives:
          - "recipient_email"
          - "email_to"
          - "to_email"
          - "destination_email"
      - name: "subject"
        description: "Email subject"
        required: true
        alternatives:
          - "email_title"
          - "mail_subject"
          - "title"
          - "email_subject"
      - name: "body"
        description: "Email content"
        required: true
        alternatives:
          - "email_body"
          - "content"
          - "message"
          - "mail_content"
          - "email_content"
      - name: "attachments"
        description: "Attachments"
        required: false
        alternatives:
          - "files"
          - "attached_files"
          - "email_attachments"

  - id: "create_ticket"
    text: "Create a new support ticket for tracking and resolving customer issues"
    description: "Create a new support ticket for tracking and resolving customer issues"
    parameters:
      - name: "title"
        description: "Ticket title"
        required: true
        alternatives:
          - "ticket_title"
          - "issue_title"
          - "ticket_name"
          - "issue_name"
      - name: "priority"
        description: "Ticket priority (urgent, normal, low)"
        required: true
        alternatives:
          - "ticket_priority"
          - "urgency"
          - "importance"
          - "severity"
      - name: "description"
        description: "Detailed problem description"
        required: true
        alternatives:
          - "ticket_description"
          - "issue_description"
          - "problem_details"
          - "details"
          - "issue_content"

  - id: "schedule_meeting"
    text: "schedule meeting"
    description: "Schedule a meeting or appointment"
    parameters:
      - name: "date"
        description: "Meeting date"
        required: true
        alternatives:
          - "meeting_date"
          - "appointment_date"
          - "scheduled_date"
          - "event_date"
      - name: "time"
        description: "Meeting time"
        required: true
        alternatives:
          - "meeting_time"
          - "appointment_time"
          - "scheduled_time"
          - "start_time"
          - "event_time"
      - name: "participants"
        description: "List of participants"
        required: true
        alternatives:
          - "attendees"
          - "meeting_participants"
          - "invitees"
          - "members"
          - "people"
      - name: "duration"
        description: "Duration in minutes"
        required: true
        alternatives:
          - "meeting_duration"
          - "length"
          - "time_duration"
          - "duration_minutes"
      - name: "topic"
        description: "Meeting topic"
        required: false
        alternatives:
          - "meeting_topic"
          - "subject"
          - "agenda"
          - "meeting_subject"

  - id: "analyze_logs"
    text: "analyze logs"
    description: "Analyze application logs"
    parameters:
      - name: "app_name"
        description: "Application name"
        required: true
        alternatives:
          - "application_name"
          - "app"
          - "application"
          - "service_name"
      - name: "start_date"
        description: "Analysis start date"
        required: true
        alternatives:
          - "from_date"
          - "begin_date"
          - "analysis_start"
          - "log_start_date"
      - name: "end_date"
        description: "Analysis end date"
        required: true
        alternatives:
          - "to_date"
          - "finish_date"
          - "analysis_end"
          - "log_end_date"
      - name: "log_level"
        description: "Log level (ERROR, WARN, INFO, DEBUG)"
        required: false
        alternatives:
          - "level"
          - "severity_level"
          - "logging_level"
          - "debug_level"

  - id: "deploy_app"
    text: "deploy application"
    description: "Deploy an application to production"
    parameters:
      - name: "app_name"
        description: "Application name to deploy"
        required: true
        alternatives:
          - "application_name"
          - "app"
          - "service_name"
          - "deployment_name"
      - name: "version"
        description: "Version to deploy"
        required: true
        alternatives:
          - "app_version"
          - "release_version"
          - "deployment_version"
          - "build_version"
      - name: "environment"
        description: "Target environment (prod, staging, dev)"
        required: true
        alternatives:
          - "env"
          - "target_env"
          - "deployment_env"
          - "target_environment"
      - name: "rollback_version"
        description: "Rollback version in case of error"
        required: false
        alternatives:
          - "backup_version"
          - "fallback_version"
          - "previous_version"
          - "revert_version"

  - id: "generate_report"
    text: "generate report"
    description: "Generate analysis or statistics report"
    parameters:
      - name: "report_type"
        description: "Report type (sales, traffic, performance)"
        required: true
        alternatives:
          - "type"
          - "kind"
          - "report_kind"
          - "analysis_type"
      - name: "period"
        description: "Report period (daily, weekly, monthly)"
        required: true
        alternatives:
          - "time_period"
          - "duration"
          - "report_period"
          - "time_range"
      - name: "format"
        description: "Output format (PDF, Excel, CSV)"
        required: true
        alternatives:
          - "output_format"
          - "file_format"
          - "report_format"
          - "export_format"

  - id: "backup_database"
    text: "backup database"
    description: "Create a database backup"
    parameters:
      - name: "database"
        description: "Database name"
        required: true
        alternatives:
          - "db_name"
          - "db"
          - "database_name"
          - "schema_name"
      - name: "backup_type"
        description: "Backup type (full, incremental)"
        required: true
        alternatives:
          - "type"
          - "backup_mode"
          - "db_backup_type"
          - "backup_method"
      - name: "compression"
        description: "Compression level (none, low, high)"
        required: false
        alternatives:
          - "compression_level"
          - "compress_level"
          - "compress_type"
          - "compression_type"

  - id: "process_payment"
    text: "process payment"
    description: "Process a customer payment"
    parameters:
      - name: "amount"
        description: "Payment amount"
        required: true
        alternatives:
          - "payment_amount"
          - "sum"
          - "total"
          - "price"
      - name: "currency"
        description: "Currency (EUR, USD)"
        required: true
        alternatives:
          - "currency_code"
          - "currency_type"
          - "payment_currency"
          - "money_type"
      - name: "payment_method"
        description: "Payment method (card, transfer, paypal)"
        required: true
        alternatives:
          - "method"
          - "pay_method"
          - "payment_type"
          - "pay_type"
      - name: "customer_id"
        description: "Customer identifier"
        required: true
        alternatives:
          - "client_id"
          - "user_id"
          - "payer_id"
          - "customer_number"
````

## File: prompts.yaml
````yaml
prompts:
  find_endpoint:
    template: |
      Given this reference sentence: '{input_sentence}'
      Compare it to these possible actions and identify which one most closely matches the core intent and meaning of the reference sentence:
      {actions_list}
      Determine the closest match by:
      1. Identifying the main verb/action in the reference sentence
      2. Extracting key elements (who, what, when, where, why, how)
      3. Comparing these elements to the fundamental purpose of each action option
      4. Selecting the action that best captures the essential meaning and purpose

      IMPORTANT: Output only the exact text of the single best matching action from the list.
      DO NOT use any markdown formatting or code blocks.
      DO NOT add any additional text or explanations.
      DO NOT wrap the response in quotes or backticks.

  match_fields:
    template: |
      Given these input fields from a sentence: '{input_fields}'\n\
      And these endpoint parameters:\n{parameters}\n\n\
      For each endpoint parameter:\n\
      1. Look at the input fields\n\
      2. Find any field that matches the parameter or its alternatives\n\
      3. Extract the actual value from the matching input field\n\n\
      Return a JSON where:\n\
      - Keys are the endpoint parameter names\n\
      - Values are the actual values found in the input fields\n\
      Only include parameters where you found a matching value.\n\
      Return valid JSON only, no additional text.

  sentence_to_json:
    template: |
      Sentence: {sentence}

      Task: Generate a precise, minimal JSON structure based strictly on the sentence.

      Rules:
      1. Create an 'endpoints' array with exactly the details from the sentence.
      2. Each endpoint must have:
         - Precise 'description' matching the sentence
         - 'fields' object where EACH key has its EXACT value from the sentence
      3. Do NOT invent additional endpoints or fields
      4. Generate only plain field with its value and not a value a field value as field and a boolean nested in
      5. Use the EXACT values found in the sentence for each field
      6. Output ONLY the valid JSON without ANY introduction sentence like here is the json
      7. Output ONLY the valid JSON without ANY explanation after outputing the json

      Example input: 'Send email to Alice at alice@example.com which title is New report and body is Hi Alice, here is the new report'
      Example output:
      Raw JSON string: {
        "endpoints": [
          {
            "id": "send_email",
            "description": "Server Down",
            "fields": {
              "to": "alice@example.com",
              "name": "Alice",
              "title": "New report"
              "content": "Hi Alice, \nhere is the new report."
            }
          }
        ]
      }

      Now for your sentence: {sentence}
````

## File: README.md
````markdown
# APICheck

A tool that uses LLM to match user inputs with predefined API endpoints based on semantic understanding.

## Prerequisites

- Rust (latest stable version)
- Ollama running locally with deepseek-r1:8b model

## Installation

1. Clone and build:
```bash
git clone git@github.com:bennekrouf/apicheck.git
cd apicheck
cargo install --path .
```

2. For local testing generate certificates:
```bash
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

## Usage

After installation, you can use `apicheck` in two ways:

### CLI Mode
```bash
# Analyze a sentence directly
apicheck "schedule a meeting tomorrow at 2pm with John"

# Show help
apicheck --help
```

### gRPC Server Mode
```bash
# Start the gRPC server (runs when no prompt is provided)
apicheck
```

## Configuration

Create a `endpoints.yaml` file in your working directory with your endpoint definitions:

```yaml
endpoints:
  - id: schedule_meeting
    text: schedule meeting
    description: Schedule a meeting with specified participants
    parameters:
      - name: time
        description: Meeting time and date
        required: true
      - name: participants
        description: List of attendees
        required: true
  - id: ....
```

## License

This project is licensed under the MIT License - see the LICENSE file for details
````

## File: sentence_service.proto
````protobuf
syntax = "proto3";

package sentence;

service SentenceService {
  rpc AnalyzeSentence (SentenceRequest) returns (stream SentenceResponse) {}
}

message SentenceRequest {
  string sentence = 1;
}

message Parameter {
  string name = 1;
  string description = 2;
  optional string semantic_value = 3;  // Added for semantic matching
}

message SentenceResponse {
  string endpoint_id = 1;
  string endpoint_description = 2;
  repeated Parameter parameters = 3;
  string json_output = 4;
}
````
