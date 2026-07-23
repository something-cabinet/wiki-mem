use serde::{Deserialize, Serialize};
use super::permission_preset_model::PermissionPreset;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PermissionsConfig {
    pub preset: PermissionPreset,
}

impl Default for PermissionsConfig {
    fn default() -> Self {
        Self {
            preset: PermissionPreset::default(),
        }
    }
}
