use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct FrEntry {
    pub id: String,
    pub description: String,
}
