use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct GoalEntry {
    pub description: String,
}
