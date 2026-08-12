//! In-process integration harness: a real `EngineState` over a tempdir
//! project with the full `ToolRegistry` wired exactly as the CLI and daemon
//! wire it. Dispatches through `ToolRegistry::dispatch_async` — the same
//! handler pipeline the transport layers exercise — so behavioral contracts
//! are covered without spawning a single subprocess.
//!
//! Some tools (`wm_index_rebuild`, `wm_project.*`) resolve paths from the
//! process CWD — the CLI chdirs to the project root before dispatch — so the
//! harness mirrors that under a process-wide guard. Callers must hold the
//! returned guard for the lifetime of the test; the guard serializes the
//! in-process tier even when the harness runs tests on parallel threads.

#[path = "setup.rs"]
pub mod setup;

use std::path::PathBuf;
use std::sync::Arc;

use wm_core::engine::EngineState;
use wm_core::mcp::tools;
use wm_core::mcp::transport::ToolRegistry;

static CWD_GUARD: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub type Ctx = (tempfile::TempDir, PathBuf, Arc<EngineState>, Arc<ToolRegistry>);

pub async fn setup_in_process() -> (Ctx, tokio::sync::MutexGuard<'static, ()>) {
    let guard = CWD_GUARD.lock().await;
    let (dir, root) = setup::setup_test_project();
    let _ = std::env::set_current_dir(&root);
    let config = wm_core::config::load_config(&root).unwrap_or_default();
    let (state, _audit_rx) = EngineState::new(config, root.clone());
    let engine = Arc::new(state);
    let mut registry = ToolRegistry::new();
    tools::register_all_tools(&mut registry, engine.clone());
    ((dir, root, engine, Arc::new(registry)), guard)
}

pub async fn call(
    registry: &ToolRegistry,
    tool: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value, wm_core::error::ToolError> {
    registry.dispatch_async(tool, args).await
}

pub async fn call_ok(
    registry: &ToolRegistry,
    tool: &str,
    args: serde_json::Value,
) -> serde_json::Value {
    call(registry, tool, args)
        .await
        .unwrap_or_else(|e| panic!("{tool} failed: {} ({})", e.message, e.code))
}

pub async fn call_err(
    registry: &ToolRegistry,
    tool: &str,
    args: serde_json::Value,
) -> wm_core::error::ToolError {
    match call(registry, tool, args).await {
        Ok(_) => panic!("{tool} unexpectedly succeeded"),
        Err(e) => e,
    }
}
