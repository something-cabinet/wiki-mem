use schemars::JsonSchema;
use serde::Deserialize;

// ─── Action enum ────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum WmTaskAction {
    #[schemars(description = "Task board grouped by status — returns full task detail per column")]
    Board {},
    #[schemars(description = "List tasks with optional filters")]
    List {
        status: Option<String>,
        priority: Option<String>,
        assignee: Option<String>,
        label: Option<String>,
        limit: Option<usize>,
    },
    #[schemars(description = "Create a task wiki page")]
    Create {
        title: String,
        description: Option<String>,
        status: Option<String>,
        priority: Option<String>,
        assignee: Option<String>,
        labels: Option<Vec<String>>,
        parent: Option<String>,
        spec: Option<String>,
        estimate: Option<u32>,
    },
    #[schemars(description = "Get a task by ID")]
    Get { id: String },
    #[schemars(description = "Update a task")]
    Update {
        id: String,
        title: Option<String>,
        status: Option<String>,
        priority: Option<String>,
        assignee: Option<String>,
        labels: Option<Vec<String>>,
        description: Option<String>,
        implementation_plan: Option<String>,
        implementation_notes: Option<String>,
        append_notes: Option<String>,
    },
    #[schemars(description = "Delete a task by ID")]
    Delete { id: String },
    #[schemars(description = "Check an acceptance criterion by index (1-based)")]
    CheckAc { id: String, index: usize },
    #[schemars(description = "Uncheck an acceptance criterion by index (1-based)")]
    UncheckAc { id: String, index: usize },
    #[schemars(description = "Create a subtask under a parent task")]
    Subtask {
        id: String,
        title: String,
        status: Option<String>,
        priority: Option<String>,
    },
}
