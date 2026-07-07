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
mod doc;
mod memory;
mod template;

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
    doc::register(registry, engine.clone());
    memory::register(registry, engine.clone());
    template::register(registry, engine);
}
