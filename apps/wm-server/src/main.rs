use std::sync::Arc;

use wm_constants::*;

mod routes;
mod server_discovery;
mod spa;
mod web_token_service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let project_root = match wm_core::config::detect_project_root() {
        Some(root) => root,
        None => anyhow::bail!("No .wm directory found in current or parent directories"),
    };

    let port = port_from_args();

    if let Some(info) = server_discovery::read_server_info(&project_root) {
        if info.port == port && server_discovery::is_running(&project_root, port) {
            anyhow::bail!(
                "wm-server is already running (pid {} on port {}). Refusing to start a duplicate daemon.",
                info.pid,
                info.port
            );
        }
    }

    let config = wm_core::config::load_config(&project_root).unwrap_or_default();
    let engine_holder = wm_core::engine::MainEngine::with_root(config, project_root.clone());
    let engine = engine_holder.state.clone();

    let mut registry = wm_core::ToolRegistry::new();
    wm_core::mcp::tools::register_all_tools(&mut registry, engine.clone());
    let registry = Arc::new(registry);

    let wiki_dir = project_root.join(WM_DIR).join(WIKI_DIR);
    if wiki_dir.exists() {
        engine_holder.rebuild_wiki(&wiki_dir);
    }

    let spa_dir = spa::find_dir(&project_root);
    let token = web_token_service::generate_and_persist(
        &project_root,
        web_token_service::TokenKind::Web,
    )?;
    let app_state = routes::AppState {
        engine: engine.clone(),
        registry,
        spa_dir: spa_dir.clone().map(Arc::new),
        token: Arc::new(token.clone()),
    };
    let api_routes = routes::build_router(app_state);
    let app = spa::build_router(api_routes, spa_dir, token);

    let addr = format!("{LOCALHOST_ADDR}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("wm-server listening on http://{addr}");

    match server_discovery::write_server_json(&project_root, port) {
        Ok(path) => tracing::info!("server discovery info written to {}", path.display()),
        Err(err) => tracing::warn!("failed to write server discovery info: {err:#}"),
    }

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
