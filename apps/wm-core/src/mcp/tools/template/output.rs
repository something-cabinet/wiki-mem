use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct WmTemplateGetOutput {
    pub name: String,
    pub description: String,
    pub content: String,
    pub variables: Vec<String>,
}

#[derive(Serialize)]
pub struct WmTemplateCreateOutput {
    pub name: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct WmTemplateRunOutput {
    pub name: String,
    pub rendered: String,
}
