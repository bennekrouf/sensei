use crate::models::Endpoint;

pub fn print_structured_analysis(
    prompt: &str,
    json_result: &serde_json::Value,
    endpoint_result: &Endpoint,
    model: &str,
) {
    println!("\nStructured Analysis (using {}):", model);
    println!("[");
    println!("  Sentence: {}", prompt);
    println!("  Matched Endpoint: {}", endpoint_result.id);

    // Get fields from JSON if available
    let binding = serde_json::Map::new();
    let fields = json_result
        .get("endpoints")
        .and_then(|endpoints| endpoints.as_array())
        .and_then(|arr| arr.first())
        .and_then(|endpoint| endpoint.get("fields"))
        .and_then(|fields| fields.as_object())
        .unwrap_or(&binding);

    // Required Parameters
    for param in endpoint_result.parameters.iter().filter(|p| p.required) {
        let value = fields
            .get(&param.name)
            .map_or("", |v| v.to_string().trim_matches('"'));
        println!("  {}: {}", param.name, value);
    }

    // Optional Parameters
    for param in endpoint_result.parameters.iter().filter(|p| !p.required) {
        let value = fields
            .get(&param.name)
            .map_or("", |v| v.to_string().trim_matches('"'));
        println!("  {}: {}", param.name, value);
    }
    println!("]\n");
}
