use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct TriggerConfig {
    pub event: String,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub priority: Option<u32>,
}
