pub struct ProviderSelector;

impl ProviderSelector {
    // Get the appropriate model name based on provider type
    pub fn get_model_name(config: &super::ModelConfig, is_claude: bool) -> String {
        if is_claude {
            if !config.claude.is_empty() {
                config.claude.clone()
            } else {
                config.name.clone() // Fallback to generic name
            }
        } else {
            if !config.ollama.is_empty() {
                config.ollama.clone()
            } else {
                config.name.clone() // Fallback to generic name
            }
        }
    }
}
