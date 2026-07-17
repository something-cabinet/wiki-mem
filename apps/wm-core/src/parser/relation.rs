use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Relation {
    #[serde(rename = "type")]
    pub edge_type: String,
    pub target: String,
}
