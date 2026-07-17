use serde::Serialize;

// ─── Output types ───────────────────────────────────────────

#[derive(Serialize)]
pub struct WmTaskCreateOutput {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub tags: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub assignee: Option<String>,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Serialize)]
pub struct WmTaskUpdateOutput {
    pub id: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct WmTaskDeleteOutput {
    pub id: String,
    pub status: String,
}
