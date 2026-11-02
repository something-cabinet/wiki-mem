use std::path::PathBuf;
use std::sync::Arc;

use wm_core::config;

use super::engine_service::create_engine_with;
use crate::shared::models::StartupRefresh;

struct EngineHandle {
    registry: Arc<wm_tools::ToolRegistry>,
}

fn engine_handle(refresh: StartupRefresh) -> anyhow::Result<Arc<EngineHandle>> {
    static HANDLE: std::sync::OnceLock<anyhow::Result<Arc<EngineHandle>>> =
        std::sync::OnceLock::new();
    match HANDLE.get_or_init(|| build_handle(refresh)) {
        Ok(arc) => Ok(Arc::clone(arc)),
        Err(e) => Err(anyhow::anyhow!("{e:#}")),
    }
}

fn build_handle(refresh: StartupRefresh) -> anyhow::Result<Arc<EngineHandle>> {
    let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
    let _ = std::env::set_current_dir(&root);
    let (engine, _) = create_engine_with(refresh);
    let mut registry = wm_tools::ToolRegistry::new();
    wm_tools::tools::register_all_tools(&mut registry, engine.state.clone());
    Ok(Arc::new(EngineHandle {
        registry: Arc::new(registry),
    }))
}

pub async fn call_tool(
    name: &str,
    arguments: serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    dispatch_tool(name, arguments, StartupRefresh::Perform).await
}

pub async fn call_rebuilding_tool(
    name: &str,
    arguments: serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    dispatch_tool(name, arguments, StartupRefresh::Skip).await
}

async fn dispatch_tool(
    name: &str,
    arguments: serde_json::Value,
    refresh: StartupRefresh,
) -> anyhow::Result<serde_json::Value> {
    let handle = engine_handle(refresh).map_err(|e| anyhow::anyhow!("{e:#}"))?;
    match handle.registry.dispatch_async(name, arguments).await {
        Ok(data) => Ok(data),
        Err(e) => Err(anyhow::anyhow!("[{}] {}", e.code, e.message)),
    }
}
