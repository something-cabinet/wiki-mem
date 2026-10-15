use crate::shared::models::TaskAction;
use crate::shared::services::call_tool;
use crate::shared::traits::Command;

const TASK_TOOL: &str = "wm_task";
const BOARD_ACTION: &str = "board";
const ACTIVE_COLUMN_LIMIT: usize = 5;
const TERMINAL_EMPTY_NOTICE: &str = "(no active tasks — use --all to see terminal columns)";
const COLLAPSED_COLUMN_SUFFIX: &str = "— use --all to list";
const COLUMN_ORDER: [&str; 10] = [
    "draft",
    "todo",
    "in-progress",
    "in-review",
    "blocked",
    "done",
    "reviewed",
    "approved",
    "superseded",
    "cancelled",
];
const TERMINAL_STATUSES: [&str; 5] = ["done", "reviewed", "approved", "superseded", "cancelled"];

impl Command for TaskAction {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_task_action(self).await
    }
}

async fn run_task_action(action: TaskAction) -> Result<(), anyhow::Error> {
    match action {
        TaskAction::Board { json } => print_task_board(json).await,
    }
}

async fn print_task_board(json: bool) -> Result<(), anyhow::Error> {
    let payload = serde_json::json!({ "action": BOARD_ACTION });
    match call_tool(TASK_TOOL, payload).await {
        Ok(board) => print_board(&board, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn print_board(board: &serde_json::Value, json: bool) -> Result<(), anyhow::Error> {
    if json {
        return print_board_json(board);
    }
    print_board_text(board);
    Ok(())
}

fn print_board_json(board: &serde_json::Value) -> Result<(), anyhow::Error> {
    let payload = serde_json::json!({
        "columns": board["columns"],
        "counts": board["counts"],
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn print_board_text(board: &serde_json::Value) {
    let columns = board["columns"].as_object().cloned().unwrap_or_default();
    let mut has_active = false;
    for column in COLUMN_ORDER {
        has_active |= print_board_column(&columns, column);
    }
    if !has_active {
        println!("{}", TERMINAL_EMPTY_NOTICE);
    }
}

fn print_board_column(columns: &serde_json::Map<String, serde_json::Value>, column: &str) -> bool {
    let Some(items) = columns.get(column).and_then(|value| value.as_array()) else {
        return false;
    };
    if items.is_empty() {
        return false;
    }
    if is_collapsed_column(column, items) {
        print_collapsed_column(column, items.len());
        return false;
    }
    print_expanded_column(column, items);
    true
}

fn is_collapsed_column(column: &str, items: &[serde_json::Value]) -> bool {
    TERMINAL_STATUSES.contains(&column) && items.len() > ACTIVE_COLUMN_LIMIT
}

fn print_collapsed_column(column: &str, count: usize) {
    println!(
        "{} ({}) {}",
        column_label(column),
        count,
        COLLAPSED_COLUMN_SUFFIX
    );
}

fn print_expanded_column(column: &str, items: &[serde_json::Value]) {
    println!("{} ({})", column_label(column), items.len());
    for item in items {
        print_board_item(item);
    }
}

fn print_board_item(item: &serde_json::Value) {
    let priority = item["priority"]
        .as_str()
        .unwrap_or(" ")
        .chars()
        .next()
        .unwrap_or(' ');
    println!("  {}  {}", priority, item["title"].as_str().unwrap_or(""));
}

fn column_label(column: &str) -> String {
    column.to_uppercase().replace('-', " ")
}
