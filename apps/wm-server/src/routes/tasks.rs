use std::sync::Arc;

use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct BoardParams {}

pub async fn board(
    State(state): State<Arc<wm_core::engine::EngineState>>,
    Json(_params): Json<BoardParams>,
) -> Json<Value> {
    let board = wm_core::task::build_task_board(&state);
    let board_value: Value = board.into();
    let mut obj = match board_value {
        Value::Object(map) => map,
        _ => serde_json::Map::new(),
    };
    obj.insert("success".into(), json!(true));
    Json(Value::Object(obj))
}
