use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use wm_core::search::QueryParams;

#[derive(Deserialize)]
pub struct QueryInput {
    pub q: String,
    #[serde(default = "default_type")]
    pub r#type: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_type() -> String {
    "all".into()
}
fn default_mode() -> String {
    "auto".into()
}
fn default_limit() -> usize {
    20
}

pub async fn query(
    State(state): State<Arc<wm_core::engine::EngineState>>,
    Json(input): Json<QueryInput>,
) -> Json<Value> {
    let params = QueryParams {
        query: input.q,
        r#type: input.r#type,
        mode: input.mode,
        limit: input.limit,
        offset: 0,
        recency: true,
    };

    match wm_core::search::run_unified_search(&state, &params) {
        Ok(resp) => {
            let items: Vec<Value> = resp
                .results
                .into_iter()
                .map(|r| {
                    let mut item = json!({
                        "id": r.id,
                        "score": r.score,
                        "type": r.r#type,
                        "page_type": r.page_type,
                        "snippet": r.snippet,
                    });
                    if let Some(sb) = r.score_breakdown {
                        item["score_breakdown"] = serde_json::to_value(sb).unwrap_or_default();
                    }
                    item
                })
                .collect();
            let mut response = json!({
                "success": true,
                "results": items,
                "total": items.len(),
            });
            if resp.degraded {
                response["degraded"] = json!(true);
                response["warning"] = json!(resp.warning);
            }
            Json(response)
        }
        Err(e) => Json(json!({"success": false, "error": e})),
    }
}

pub async fn retrieve(
    State(_state): State<Arc<wm_core::engine::EngineState>>,
    Json(input): Json<Value>,
) -> Json<Value> {
    let _q = input.get("q").and_then(|v| v.as_str()).unwrap_or("");
    Json(json!({"success": true, "context": ""}))
}
