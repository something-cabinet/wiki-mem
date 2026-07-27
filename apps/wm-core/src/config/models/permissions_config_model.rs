use super::permission_preset_model::PermissionPreset;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PermissionsConfig {
    pub preset: PermissionPreset,
}
