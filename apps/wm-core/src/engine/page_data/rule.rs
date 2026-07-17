use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuleCategory {
    Naming,
    Branching,
    Design,
    ModuleStructure,
    ErrorHandling,
    DataModeling,
    Concurrency,
    Testing,
    Operational,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleData {
    pub category: RuleCategory,
    pub rationale: String,
    pub example: Option<String>,
    pub anti_pattern: Option<String>,
}
