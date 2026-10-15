use std::path::Path;
use std::sync::Arc;

use crate::shared::helpers::determine_project_root;
use crate::shared::models::{McpArgs, McpTransport};
use crate::shared::services::build_engine;
use crate::shared::traits::Command;
use wm_core::engine::EngineState;
use wm_core::ToolRegistry;

impl Command for McpArgs {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_mcp(self).await
    }
}

async fn run_mcp(args: McpArgs) -> Result<(), anyhow::Error> {
    let project_root = determine_project_root(&args.project)?;
    match args.transport {
        McpTransport::Stdio => serve_stdio(&project_root).await,
    }
}

async fn serve_stdio(project_root: &Path) -> Result<(), anyhow::Error> {
    enter_project_dir(project_root)?;
    tracing::info!("MCP server starting (project {})", project_root.display());
    let engine = build_engine(project_root);
    let registry = build_registry(engine.state.clone());
    crate::mcp_server::serve(registry).await
}

fn enter_project_dir(project_root: &Path) -> Result<(), anyhow::Error> {
    std::env::set_current_dir(project_root).map_err(|error| {
        anyhow::anyhow!(
            "failed to change to project dir {}: {error}",
            project_root.display()
        )
    })
}

fn build_registry(state: Arc<EngineState>) -> Arc<ToolRegistry> {
    let mut registry = ToolRegistry::new();
    wm_core::mcp::tools::register_all_tools(&mut registry, state);
    Arc::new(registry)
}
