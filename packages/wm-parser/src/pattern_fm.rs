use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct PatternFm {
    pub when_to_use: String,
    pub example: String,
}
