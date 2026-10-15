use std::path::PathBuf;
use std::sync::Arc;

use wm_core::config;

use super::engine_service::create_engine;

struct EngineHandle {
    registry: Arc<wm_core::ToolRegistry>,
}

fn engine_handle() -> anyhow::Result<Arc<EngineHandle>> {
    static HANDLE: std::sync::OnceLock<anyhow::Result<Arc<EngineHandle>>> =
        std::sync::OnceLock::new();
    match HANDLE.get_or_init(build_handle) {
        Ok(arc) => Ok(Arc::clone(arc)),
        Err(e) => Err(anyhow::anyhow!("{e:#}")),
    }
}

fn build_handle() -> anyhow::Result<Arc<EngineHandle>> {
    let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
    let _ = std::env::set_current_dir(&root);
    let (engine, _) = create_engine();
    let mut registry = wm_core::ToolRegistry::new();
    wm_core::mcp::tools::register_all_tools(&mut registry, engine.state.clone());
    Ok(Arc::new(EngineHandle {
        registry: Arc::new(registry),
    }))
}

pub async fn call_tool(
    name: &str,
    arguments: serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    let handle = engine_handle().map_err(|e| anyhow::anyhow!("{e:#}"))?;
    match handle.registry.dispatch_async(name, arguments).await {
        Ok(data) => Ok(data),
        Err(e) => Err(anyhow::anyhow!("[{}] {}", e.code, e.message)),
    }
}
