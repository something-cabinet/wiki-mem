use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum MemoryLayer {
    #[default]
    Project,
    Global,
    Session,
}

impl MemoryLayer {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryLayer::Project => "project",
            MemoryLayer::Global => "global",
            MemoryLayer::Session => "session",
        }
    }
}
