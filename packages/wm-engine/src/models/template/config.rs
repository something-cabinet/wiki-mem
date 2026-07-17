use serde::{Deserialize, Serialize};

use super::prompt::TemplatePrompt;
use super::action::TemplateAction;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub name: String,
    pub description: String,
    pub doc: Option<String>,
    pub destination: Option<String>,
    pub prompts: Vec<TemplatePrompt>,
    pub actions: Vec<TemplateAction>,
    pub messages: Option<serde_json::Value>,
}
