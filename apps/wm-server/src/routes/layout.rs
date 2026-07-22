use std::sync::Arc;

use axum::extract::State;
use serde_json::json;
use wm_core::engine::EngineState;

/// Layout computation has moved to the browser via fjadra WASM.
/// This file only retains the SSE streaming endpoint for backward
/// compatibility. See packages/fjadra-wasm/ for the current implementation.

/// `GET /api/graph/layout/{job_id}/events` – SSE stream of layout events (stub for future).
///
/// The current implementation returns positions directly from POST. This endpoint
/// exists for future two-phase streaming support but currently returns final positions
/// as a single `graph-settled` event.
pub async fn stream_events(
    State(state): State<Arc<EngineState>>,
    axum::extract::Path(_job_id): axum::extract::Path<String>,
) -> axum::response::Sse<impl tokio_stream::Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>> {
    use axum::response::sse::Event;
    use futures::stream;

    let snapshot = state.graph.load();
    let graph = &snapshot.0;

    let mut positions: Vec<[f64; 2]> = Vec::new();
    for node_idx in graph.node_indices() {
        let i = node_idx.index() as f64;
        let angle = i * 2.399;
        let radius = 200.0 + (i * 50.0).sqrt();
        positions.push([radius * angle.cos(), radius * angle.sin()]);
    }

    let positions_json = serde_json::to_string(&json!({"positions": positions})).unwrap_or_default();

    let stream = stream::once(async move {
        Ok::<_, std::convert::Infallible>(Event::default()
            .event("graph-settled")
            .data(positions_json))
    });

    axum::response::Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(5))
            .text("keep-alive"),
    )
}
