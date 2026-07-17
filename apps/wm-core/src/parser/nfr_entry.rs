use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct NfrEntry {
    pub id: String,
    pub description: String,
}
