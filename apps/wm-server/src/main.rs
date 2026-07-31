use std::sync::Arc;

use wm_constants::*;

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

    let wiki_dir = project_root.join(WM_DIR).join(WIKI_DIR);
    if wiki_dir.exists() {
        engine.rebuild_graph(&wiki_dir);
    }

    let spa_dir = spa::find_dir(&project_root);
    let app_state = routes::AppState {
        engine: engine.clone(),
        registry,
        spa_dir: spa_dir.clone().map(Arc::new),
    };
    let api_routes = routes::build_router(app_state);
    let app = spa::build_router(api_routes, spa_dir);

    let port = port_from_args();
    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("wm-server listening on http://{addr}");

    axum::serve(listener, app).await?;
    Ok(())
}

fn port_from_args() -> u16 {
    let args: Vec<String> = std::env::args().collect();
    let mut port = DEFAULT_PORT;
    let mut i = 0;
    while i + 1 < args.len() {
        if args[i] == "--port" {
            if let Ok(p) = args[i + 1].parse::<u16>() {
                port = p;
            }
            break;
        }
        i += 1;
    }
    port
}
