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
