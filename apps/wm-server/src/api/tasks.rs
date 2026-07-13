use axum::{extract::State, Json};
use serde_json::{json, Value};
use wm_core::task;
use crate::AppState;

pub async fn task_board(
    State(state): State<AppState>,
) -> Json<Value> {
    let board = task::build_task_board(&state.engine);
    Json(json!({
        "success": true,
        "board": board,
    }))
}
