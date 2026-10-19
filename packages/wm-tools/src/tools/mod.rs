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
mod record_format_hint;
mod record_validation;
mod reference;
mod search;
mod skills;
mod source;
pub mod task;
pub mod template;
mod time;
mod validate;
mod version;

use crate::transport::ToolRegistry;
use std::sync::Arc;
use wm_core::engine::EngineState;

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
    let tool_list = registry.tool_list_slot();
    project::register(registry, engine.clone(), tool_list);
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

    registry.freeze_tool_list();

    if let Ok(triggered) =
        skills::fire_session_event(&engine, &wm_core::skill::TriggerEvent::SessionStart)
    {
        if triggered["count"].as_u64().unwrap_or(0) > 0 {
            tracing::info!("SessionStart triggered {} skill(s)", triggered["count"]);
        }
    }
}
