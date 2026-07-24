use std::sync::Arc;

mod engine;
mod routes;
mod server_discovery;
mod spa;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let project_root = match wm_core::config::detect_project_root() {
        Some(root) => root,
        None => anyhow::bail!("No .wm directory found in current or parent directories"),
    };

    let config = wm_core::config::load_config(&project_root).unwrap_or_default();
    let (engine_state, audit_rx) = wm_core::engine::EngineState::new(config, project_root.clone());
    let engine = Arc::new(engine_state);

    let mut registry = wm_core::ToolRegistry::new();
    wm_core::mcp::tools::register_all_tools(&mut registry, engine.clone());
    let registry = Arc::new(registry);

    tokio::spawn(async move {
        let mut rx = audit_rx;
        while rx.recv().await.is_some() {}
    });

    let wiki_dir = project_root.join(".wm").join("wiki");
    if wiki_dir.exists() {
        engine.rebuild_graph(&wiki_dir);
    }

    let app_state = routes::AppState {
        engine: engine.clone(),
        registry,
    };
    let app = routes::build_router(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4090").await?;
    tracing::info!("wm-server listening on http://127.0.0.1:4090");

    axum::serve(listener, app).await?;
    Ok(())
}
