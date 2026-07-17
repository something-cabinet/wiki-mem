use serde::Deserialize;

use super::trigger_config_model::TriggerConfig;

#[derive(Debug, Deserialize)]
pub(crate) struct SkillFrontmatter {
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub trigger: Option<TriggerConfig>,
}
