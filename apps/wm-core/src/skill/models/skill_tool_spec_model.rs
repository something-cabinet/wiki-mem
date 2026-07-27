use std::sync::Arc;

pub struct SkillToolSpec {
    pub name: String,
    pub description: String,
    pub handler: Arc<
        dyn Fn(serde_json::Value) -> Result<serde_json::Value, crate::error::ToolError>
            + Send
            + Sync,
    >,
}
