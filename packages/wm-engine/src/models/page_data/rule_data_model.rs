use serde::{Deserialize, Serialize};

use super::rule_category_model::RuleCategory;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleData {
    pub category: RuleCategory,
    pub rationale: String,
    pub example: Option<String>,
    pub anti_pattern: Option<String>,
}
