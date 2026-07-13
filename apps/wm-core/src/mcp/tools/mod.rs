// Schema-only fields on MCP tool inputs are read by schemars for JSON schema
// generation but not always accessed by handler code — suppress the warning.
#![allow(dead_code)]

// ─── MCP Tool Delegator ─────────────────────────────────────
// Each domain module exposes a `pub fn register(registry, engine)`.

mod search;
mod page;
mod source;
mod graph;
mod lint;
mod validate;
mod index;
mod task;
mod log;
mod model;
mod time;
mod project;
mod skills;
mod reference;
mod decision;
mod doc;
mod memory;
mod template;
mod code;

use std::sync::Arc;
use crate::engine::EngineState;
use crate::mcp::transport::ToolRegistry;

/// Register all MCP tool handlers by delegating to domain modules.
pub fn register_all_tools(
    registry: &mut ToolRegistry,
    engine: Arc<EngineState>,
) {
    search::register(registry, engine.clone());
    page::register(registry, engine.clone());
    source::register(registry, engine.clone());
    graph::register(registry, engine.clone());
    lint::register(registry, engine.clone());
    validate::register(registry, engine.clone());
    index::register(registry, engine.clone());
    task::register(registry, engine.clone());
    log::register(registry, engine.clone());
    model::register(registry, engine.clone());
    time::register(registry, engine.clone());
    project::register(registry, engine.clone());
    skills::register(registry, engine.clone());
    reference::register(registry, engine.clone());
    decision::register(registry, engine.clone());
    doc::register(registry, engine.clone());
    memory::register(registry, engine.clone());
    template::register(registry, engine.clone());
    code::register(registry, engine.clone());

    // Fire SessionStart lifecycle event on MCP server startup
    if let Ok(triggered) = skills::fire_session_event(&engine, &crate::skill::TriggerEvent::SessionStart) {
        if triggered["count"].as_u64().unwrap_or(0) > 0 {
            tracing::info!("SessionStart triggered {} skill(s)", triggered["count"]);
        }
    }
}
