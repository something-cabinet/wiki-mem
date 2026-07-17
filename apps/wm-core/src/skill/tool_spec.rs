use std::sync::Arc;

pub struct SkillToolSpec {
    pub name: String,
    pub description: String,
    pub handler: Arc<dyn Fn(serde_json::Value) -> Result<serde_json::Value, wm_error::ToolError> + Send + Sync>,
}
