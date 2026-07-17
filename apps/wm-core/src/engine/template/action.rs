use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemplateAction {
    pub r#type: String,       // "add", "addMany", "modify", "append"
    pub template: Option<String>,
    pub path: String,
    pub source: Option<String>,
    pub skip_if_exists: Option<bool>,
    pub when: Option<String>,
    pub insert_after: Option<String>,
}
