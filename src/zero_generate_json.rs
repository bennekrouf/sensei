use crate::call_ollama::call_ollama;
use regex::Regex;
use std::error::Error;
use tracing::{debug, error, info};

pub async fn generate_json(
    model: &str,
    sentence: &str,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let full_prompt = format!(
        "Sentence: {}\n\n\
        Task: Generate a precise, minimal JSON structure based strictly on the sentence.\n\n\
        Rules:\n\
        1. Create an 'endpoints' array with exactly the details from the sentence.\n\
        2. Each endpoint must have:\n\
           - Precise 'description' matching the sentence\n\
           - 'fields' object where EACH key has its EXACT value from the sentence\n\
        3. Do NOT invent additional endpoints or fields\n\
        4. Generate only plain field with its value and not a value a field value as field and a boolean nested in\n\
        5. Use the EXACT values found in the sentence for each field\n\
        6. Output ONLY the valid JSON without ANY introduction sentence like here is the json\n\
        7. Output ONLY the valid JSON without ANY explanation after outputing the json\n\n\
        Example input: 'Send email to Alice at alice@example.com'\n\
        Example output:\n\
        {{\n\
          \"endpoints\": [\n\
            {{\n\
              \"description\": \"Send email\",\n\
              \"fields\": {{\n\
                \"recipient\": \"Alice\",\n\
                \"email\": \"alice@example.com\"\n\
              }}\n\
            }}\n\
          ]\n\
        }}\n\n\
        Now for your sentence: {}",
        sentence, sentence
    );

    let full_response_text = call_ollama(&model, &full_prompt).await?;
    debug!("Raw LLM response:\n{}", full_response_text);

    // Extract JSON using regex
    let re = Regex::new(r"\{[\s\S]*\}")?;
    let json_str = re
        .find(&full_response_text)
        .ok_or_else(|| {
            error!("No JSON found in response: {}", full_response_text);
            "No JSON structure found in response"
        })?
        .as_str();

    debug!("Extracted JSON string:\n{}", json_str);

    // Attempt to parse the JSON
    let parsed_json: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
        error!("Failed to parse JSON: {}\nRaw JSON string: {}", e, json_str);
        format!("Failed to parse JSON: {}. Raw JSON: {}", e, json_str)
    })?;

    // Validate the JSON structure
    if !parsed_json.is_object() || !parsed_json.get("endpoints").is_some() {
        error!("Invalid JSON structure: missing 'endpoints' array");
        return Err("Invalid JSON structure: missing 'endpoints' array".into());
    }

    info!("Successfully generated and validated JSON");
    Ok(parsed_json)
}
