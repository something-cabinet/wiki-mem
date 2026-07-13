// ─── wm-server: HTTP API Server ───────────────────────
//
// Owns the engine (graph, BM25, memory, embedder).
// Exposes REST API for all operations.
// wm-mcp and Angular UI both talk to this server.

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting wm-server");

    // TODO: Own the EngineState, expose all operations via REST API
    // Routes for: search, pages, tasks, memory, graph, time, index,
    // models, lint, validate, logs, project, skills, refs, decisions,
    // docs, templates, code-intel

    Ok(())
}
