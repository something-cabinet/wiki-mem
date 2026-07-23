use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectionDoc {
    pub section_id: String,
    pub page_id: String,
    pub header: String,
    pub body: String,
    pub title: String,
    pub tags: Vec<String>,
}
