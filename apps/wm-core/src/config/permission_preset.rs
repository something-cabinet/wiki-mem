use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PermissionPreset {
    ReadWrite,
    ReadOnly,
}

impl Default for PermissionPreset {
    fn default() -> Self {
        PermissionPreset::ReadWrite
    }
}
