use crate::lint::LintAction;
use crate::shared::services::call_tool;
use crate::shared::traits::Command;

const GRAPH_STATS_TOOL: &str = "wm_graph.stats";
const LINT_CHECK_TOOL: &str = "wm_lint.check";
const LINT_FIX_TOOL: &str = "wm_lint.fix";
const ORPHAN_ISSUE_TYPE: &str = "orphan";

impl Command for LintAction {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_lint_action(self).await
    }
}

async fn run_lint_action(action: LintAction) -> Result<(), anyhow::Error> {
    match action {
        LintAction::Check { json } => check_lint(json).await,
        LintAction::Fix { json } => fix_lint(json).await,
    }
}

async fn check_lint(json: bool) -> Result<(), anyhow::Error> {
    let stats = call_tool(GRAPH_STATS_TOOL, serde_json::json!({})).await;
    let lint = call_tool(LINT_CHECK_TOOL, serde_json::json!({})).await;
    match (stats, lint) {
        (Ok(stats), Ok(lint)) => print_lint_check(&stats, &lint, json)?,
        (Err(error), _) => eprintln!("Error: {}", error),
        (_, Err(error)) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn print_lint_check(
    stats: &serde_json::Value,
    lint: &serde_json::Value,
    json: bool,
) -> Result<(), anyhow::Error> {
    let nodes = stats["nodes"].as_u64().unwrap_or(0);
    let edges = stats["edges"].as_u64().unwrap_or(0);
    let orphans = count_orphan_issues(lint);
    if json {
        return print_lint_check_json(nodes, edges, orphans);
    }
    print_lint_check_text(nodes, edges, orphans);
    Ok(())
}

fn print_lint_check_json(nodes: u64, edges: u64, orphans: usize) -> Result<(), anyhow::Error> {
    let payload = serde_json::json!({
        "nodes": nodes, "edges": edges, "orphans": orphans
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn print_lint_check_text(nodes: u64, edges: u64, orphans: usize) {
    println!("Lint check complete:");
    println!("  Nodes: {}", nodes);
    println!("  Edges: {}", edges);
    println!("  Orphans: {}", orphans);
}

fn count_orphan_issues(lint: &serde_json::Value) -> usize {
    lint["issues"]
        .as_array()
        .map(|issues| {
            issues
                .iter()
                .filter(|issue| issue["type"].as_str() == Some(ORPHAN_ISSUE_TYPE))
                .count()
        })
        .unwrap_or(0)
}

async fn fix_lint(json: bool) -> Result<(), anyhow::Error> {
    match call_tool(LINT_FIX_TOOL, serde_json::json!({})).await {
        Ok(response) => print_lint_fix(&response, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn print_lint_fix(response: &serde_json::Value, json: bool) -> Result<(), anyhow::Error> {
    let fixed = response["fixed"].as_u64().unwrap_or(0);
    if json {
        let payload = serde_json::json!({ "fixed": fixed });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }
    println!("Fixed {} issue(s)", fixed);
    Ok(())
}
