use std::path::PathBuf;

use super::trigger_config::TriggerConfig;

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub title: String,
    pub description: String,
    pub trigger: Option<TriggerConfig>,
    pub instructions: String,
    pub file_path: PathBuf,
}
