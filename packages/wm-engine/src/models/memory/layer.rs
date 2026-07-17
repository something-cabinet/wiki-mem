use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MemoryLayer {
    Project,
    Global,
    Session,
}

impl Default for MemoryLayer {
    fn default() -> Self { MemoryLayer::Project }
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
