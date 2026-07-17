use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NonFunctionalRequirement {
    pub id: String,
    pub description: String,
}
