mod code;
mod decision;
mod doc;
mod graph;
mod index;
mod lint;
mod log;
#[cfg(feature = "lsp")]
pub mod lsp;
mod memory;
pub mod model;
pub mod page;
mod project;
mod reference;
mod search;
mod skills;
mod source;
pub mod task;
pub mod template;
mod time;
mod validate;
mod version;

use crate::engine::EngineState;
use crate::mcp::transport::ToolRegistry;
use std::sync::Arc;

pub fn register_all_tools(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
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
    memory::register(registry, engine.clone());
    template::register(registry, engine.clone());
    doc::register(registry, engine.clone());
    code::register(registry, engine.clone());
    version::register(registry, engine.clone());

    #[cfg(feature = "lsp")]
    lsp::register(registry, engine.clone());

    engine.set_tool_list(registry.list_tools());

    if let Ok(triggered) =
        skills::fire_session_event(&engine, &crate::skill::TriggerEvent::SessionStart)
    {
        if triggered["count"].as_u64().unwrap_or(0) > 0 {
            tracing::info!("SessionStart triggered {} skill(s)", triggered["count"]);
        }
    }
}
