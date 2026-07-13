use std::collections::HashMap;
use serde_json::Value;
use crate::engine::{EngineState, PageType, PageStatus, Priority};

/// A single task item for the board
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TaskBoardItem {
    pub id: String,
    pub title: String,
    pub priority: String,
}

/// Task board grouped by status columns
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TaskBoard {
    pub columns: HashMap<String, Vec<TaskBoardItem>>,
    pub counts: HashMap<String, usize>,
}

/// Build a task board by iterating the graph, binning tasks by status.
/// Returns a structured TaskBoard that both CLI and MCP can format.
pub fn build_task_board(engine: &EngineState) -> TaskBoard {
    let snapshot = engine.graph.load();
    let graph = &snapshot.0;

    let all_statuses = PageStatus::task_board_columns();

    // Initialize buckets for each status
    let mut buckets: HashMap<String, Vec<TaskBoardItem>> = HashMap::new();
    for status in &all_statuses {
        buckets.insert(status.as_str().to_string(), Vec::new());
    }

    for idx in graph.node_indices() {
        let meta = &graph[idx];
        if meta.page_type != PageType::Task {
            continue;
        }
        let entry = TaskBoardItem {
            id: meta.id.clone(),
            title: meta.title.clone(),
            priority: meta
                .priority
                .as_ref()
                .unwrap_or(&Priority::Medium)
                .as_str()
                .to_string(),
        };
        let key = meta.status.as_str().to_string();
        buckets.entry(key).or_default().push(entry);
    }

    let mut columns = HashMap::new();
    let mut counts = HashMap::new();
    for status in &all_statuses {
        let key = status.as_str().to_string();
        let items = buckets.remove(&key).unwrap_or_default();
        columns.insert(key.clone(), items.clone());
        counts.insert(key, items.len());
    }

    TaskBoard { columns, counts }
}

impl From<TaskBoard> for Value {
    fn from(board: TaskBoard) -> Self {
        serde_json::json!({
            "columns": board.columns,
            "counts": board.counts,
        })
    }
}
