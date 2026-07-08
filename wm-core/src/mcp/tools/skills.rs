use std::sync::Arc;

use serde_json::json;

use crate::engine::EngineState;
use crate::mcp::transport::ToolRegistry;

/// Register skill tool handlers (inverted dependency: skill → MCP)
pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    if let Ok(skill_engine) = engine.skill_engine.read() {
        for spec in skill_engine.tool_specs() {
            registry.register_with_schema(&spec.name, &spec.description, json!({
                "type": "object",
                "properties": {}
            }), spec.handler);
        }
    }
}
