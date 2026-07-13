use std::sync::Arc;
use tracing::info;
use wm_core::engine::EngineState;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = wm_core::config::ProjectConfig::default();
    let (engine_state, _audit_rx) = EngineState::new(config);
    let engine = Arc::new(engine_state);

    let port = std::env::var("WM_WEB_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000u16);

    info!("WM Web Server starting on port {}", port);
    wm_web::run_server(engine, port).await
}
