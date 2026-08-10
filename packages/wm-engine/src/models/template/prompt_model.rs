use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplatePrompt {
    pub name: String,
    pub r#type: String,
    pub message: String,
    pub initial: Option<serde_json::Value>,
    pub validate: Option<String>,
    pub choices: Option<Vec<String>>,
}
