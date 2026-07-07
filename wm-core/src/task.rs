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
pub fn task_board(engine: &EngineState) -> TaskBoard {
    let snapshot = engine.graph.load();
    let graph = &snapshot.0;

    let mut todo = Vec::new();
    let mut in_progress = Vec::new();
    let mut done = Vec::new();
    let mut blocked = Vec::new();

    for idx in graph.node_indices() {
        let meta = &graph[idx];
        if meta.page_type != PageType::Task {
            continue;
        }
        let entry = TaskBoardItem {
            id: meta.id.clone(),
            title: meta.title.clone(),
            priority: format!("{:?}", meta.priority.as_ref().unwrap_or(&Priority::Medium)).to_lowercase(),
        };
        match meta.status {
            PageStatus::Todo => todo.push(entry),
            PageStatus::InProgress => in_progress.push(entry),
            PageStatus::Done => done.push(entry),
            PageStatus::Blocked => blocked.push(entry),
            _ => todo.push(entry),
        }
    }

    let mut columns = HashMap::new();
    columns.insert("todo".to_string(), todo.clone());
    columns.insert("in_progress".to_string(), in_progress.clone());
    columns.insert("done".to_string(), done.clone());
    columns.insert("blocked".to_string(), blocked.clone());

    let mut counts = HashMap::new();
    counts.insert("todo".to_string(), todo.len());
    counts.insert("in_progress".to_string(), in_progress.len());
    counts.insert("done".to_string(), done.len());
    counts.insert("blocked".to_string(), blocked.len());

    TaskBoard {
        columns,
        counts,
    }
}

impl From<TaskBoard> for Value {
    fn from(board: TaskBoard) -> Self {
        serde_json::json!({
            "columns": board.columns,
            "counts": board.counts,
        })
    }
}
