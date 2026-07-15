use axum::{
    extract::State,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use wm_core::search::{self, QueryParams};
use crate::AppState;

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
    #[serde(rename = "type")]
    r#type: Option<String>,
    mode: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

pub async fn handle_search(
    State(state): State<AppState>,
    Json(params): Json<SearchQuery>,
) -> Json<Value> {
    let qp = QueryParams {
        query: params.q.clone(),
        r#type: params.r#type.unwrap_or_else(|| "all".into()),
        mode: params.mode.unwrap_or_else(|| "auto".into()),
        limit: params.limit.unwrap_or(20),
        offset: params.offset.unwrap_or(0),
        recency: true,
    };
    match search::query::run_unified_search(&state.engine, &qp) {
        Ok(results) => {
            let items: Vec<Value> = results
                .iter()
                .map(|r| {
                    json!({
                        "id": r.id,
                        "score": r.score,
                        "type": r.r#type,
                        "page_type": r.page_type,
                        "page_type_rank": r.page_type_rank,
                        "centrality": r.centrality,
                        "snippet": r.snippet,
                    })
                })
                .collect();
            Json(json!({
                "success": true,
                "results": items,
                "total": items.len(),
            }))
        }
        Err(e) => Json(json!({
            "success": false,
            "error": e,
        })),
    }
}
