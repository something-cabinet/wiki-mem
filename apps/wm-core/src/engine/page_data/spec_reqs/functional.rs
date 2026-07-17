use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FunctionalRequirement {
    pub id: String,
    pub description: String,
}
