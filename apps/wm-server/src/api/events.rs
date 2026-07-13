use axum::{
    extract::State,
    response::sse::{Event, Sse},
};
use futures::stream::Stream;
use serde_json::json;
use std::convert::Infallible;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use crate::AppState;

pub async fn event_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = mpsc::channel::<String>(64);

    let engine = state.engine.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            let snapshot = engine.graph.load();
            let page_count = snapshot.0.node_count();
            let stats = json!({
                "type": "heartbeat",
                "page_count": page_count,
                "memory_count": engine.session_memory.len(),
            });
            if tx.send(stats.to_string()).await.is_err() {
                break;
            }
        }
    });

    let stream = ReceiverStream::new(rx).map(|data| {
        Ok(Event::default().event("state").data(data))
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}
