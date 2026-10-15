use crate::shared::models::LogAction;
use crate::shared::traits::Command;
use wm_constants::{LOG_FILE, WM_DIR};

impl Command for LogAction {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_log_action(self)
    }
}

fn run_log_action(action: LogAction) -> Result<(), anyhow::Error> {
    match action {
        LogAction::Recent { count, json } => print_recent_entries(count, json),
        LogAction::Since { marker, json } => print_entries_since(&marker, json),
        LogAction::Filter { text, json } => print_filtered_entries(&text, json),
    }
}

fn print_recent_entries(count: usize, json: bool) -> Result<(), anyhow::Error> {
    let lines = non_empty_log_lines();
    let start = lines.len().saturating_sub(count);
    print_log_entries(&lines[start..], lines.len(), json)
}

fn print_entries_since(marker: &str, json: bool) -> Result<(), anyhow::Error> {
    let lines = read_log_lines();
    let entries = log_entries_since(&lines, marker);
    print_log_entries(&entries, entries.len(), json)
}

fn print_filtered_entries(text: &str, json: bool) -> Result<(), anyhow::Error> {
    let lines = read_log_lines();
    let entries = log_entries_matching(&lines, text);
    print_log_entries(&entries, entries.len(), json)
}

fn read_log_lines() -> Vec<String> {
    let log_path = std::path::Path::new(WM_DIR).join(LOG_FILE);
    let content = std::fs::read_to_string(&log_path).unwrap_or_default();
    content.lines().map(str::to_owned).collect()
}

fn non_empty_log_lines() -> Vec<String> {
    read_log_lines()
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .collect()
}

fn log_entries_since(lines: &[String], marker: &str) -> Vec<String> {
    lines
        .iter()
        .skip_while(|line| !line.contains(marker))
        .skip(1)
        .cloned()
        .collect()
}

fn log_entries_matching(lines: &[String], text: &str) -> Vec<String> {
    let needle = text.to_lowercase();
    lines
        .iter()
        .filter(|line| line.to_lowercase().contains(&needle))
        .cloned()
        .collect()
}

fn print_log_entries(entries: &[String], total: usize, json: bool) -> Result<(), anyhow::Error> {
    if json {
        let payload = serde_json::json!({ "entries": entries, "total": total });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }
    for entry in entries {
        println!("{}", entry);
    }
    Ok(())
}
