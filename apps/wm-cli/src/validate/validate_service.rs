use crate::validate::ValidateArgs;
use crate::shared::services::call_tool;
use crate::shared::traits::Command;

const VALIDATE_TOOL: &str = "wm_validate.check";
const STATUS_PASS: &str = "pass";
const STATUS_EMPTY: &str = "empty";

impl Command for ValidateArgs {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_validation(self.json).await
    }
}

async fn run_validation(json: bool) -> Result<(), anyhow::Error> {
    match call_tool(VALIDATE_TOOL, serde_json::json!({})).await {
        Ok(response) => print_validation(&response, json),
        Err(error) => {
            eprintln!("Error: {}", error);
            Ok(())
        }
    }
}

fn print_validation(response: &serde_json::Value, json: bool) -> Result<(), anyhow::Error> {
    let nodes = response["nodes"].as_u64().unwrap_or(0);
    let edges = response["edges"].as_u64().unwrap_or(0);
    if json {
        return print_validation_json(nodes, edges);
    }
    print_validation_text(nodes, edges);
    Ok(())
}

fn print_validation_json(nodes: u64, edges: u64) -> Result<(), anyhow::Error> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "nodes": nodes,
            "edges": edges,
            "status": validation_status(nodes),
        }))?
    );
    Ok(())
}

fn print_validation_text(nodes: u64, edges: u64) {
    println!("Validation complete: {} nodes, {} edges", nodes, edges);
    if nodes > 0 {
        println!("Status: {}", STATUS_PASS);
    }
}

fn validation_status(nodes: u64) -> &'static str {
    if nodes > 0 {
        return STATUS_PASS;
    }
    STATUS_EMPTY
}
