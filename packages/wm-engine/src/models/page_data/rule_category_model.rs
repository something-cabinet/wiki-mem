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
