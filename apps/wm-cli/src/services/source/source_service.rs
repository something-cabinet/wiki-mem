use crate::shared::models::SourceAction;
use crate::shared::services::call_tool;
use crate::shared::traits::Command;

const SOURCE_TOOL: &str = "wm_source";
const ACTION_LIST: &str = "list";
const ACTION_STATUS: &str = "status";
const ACTION_REMOVE: &str = "remove";
const ACTION_DISCOVER: &str = "discover";

impl Command for SourceAction {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_source_action(self).await
    }
}

async fn run_source_action(action: SourceAction) -> Result<(), anyhow::Error> {
    match action {
        SourceAction::List { state, json } => list_sources(state, json).await,
        SourceAction::Status { id, json } => show_source_status(&id, json).await,
        SourceAction::Remove { id, .. } => remove_source(&id).await,
        SourceAction::Discover { json } => discover_sources(json).await,
    }
}

async fn list_sources(state: Option<String>, json: bool) -> Result<(), anyhow::Error> {
    let mut args = serde_json::Map::new();
    args.insert("action".into(), serde_json::json!(ACTION_LIST));
    if let Some(state) = state {
        args.insert("state".into(), serde_json::json!(state));
    }
    match call_tool(SOURCE_TOOL, serde_json::Value::Object(args)).await {
        Ok(response) => print_sources(&response, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn print_sources(response: &serde_json::Value, json: bool) -> Result<(), anyhow::Error> {
    let sources = response["sources"].as_array().cloned().unwrap_or_default();
    if json {
        return print_source_list_json(&sources);
    }
    print_source_list_text(&sources);
    Ok(())
}

fn print_source_list_json(sources: &[serde_json::Value]) -> Result<(), anyhow::Error> {
    let payload = serde_json::json!({ "sources": sources, "total": sources.len() });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn print_source_list_text(sources: &[serde_json::Value]) {
    for source in sources {
        println!("  {}  [{:?}]", source["id"], source["state"]);
    }
    println!("{} sources", sources.len());
}

async fn show_source_status(id: &str, json: bool) -> Result<(), anyhow::Error> {
    let payload = serde_json::json!({ "action": ACTION_STATUS, "id": id });
    match call_tool(SOURCE_TOOL, payload).await {
        Ok(status) => print_source_status(&status, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn print_source_status(status: &serde_json::Value, json: bool) -> Result<(), anyhow::Error> {
    if json {
        println!("{}", serde_json::to_string_pretty(status)?);
        return Ok(());
    }
    println!("ID:       {}", status["id"]);
    println!("State:    {}", status["state"]);
    println!("Pages:    {}", status["page_count"]);
    Ok(())
}

async fn remove_source(id: &str) -> Result<(), anyhow::Error> {
    let payload = serde_json::json!({ "action": ACTION_REMOVE, "id": id });
    match call_tool(SOURCE_TOOL, payload).await {
        Ok(_) => println!("Removed source: {}", id),
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

async fn discover_sources(json: bool) -> Result<(), anyhow::Error> {
    let payload = serde_json::json!({ "action": ACTION_DISCOVER });
    match call_tool(SOURCE_TOOL, payload).await {
        Ok(response) => print_discovered_sources(&response, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn print_discovered_sources(response: &serde_json::Value, json: bool) -> Result<(), anyhow::Error> {
    let discovered = response["discovered"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if json {
        return print_discovered_json(&discovered);
    }
    print_discovered_text(&discovered);
    Ok(())
}

fn print_discovered_json(discovered: &[serde_json::Value]) -> Result<(), anyhow::Error> {
    let payload = serde_json::json!({ "discovered": discovered, "total": discovered.len() });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn print_discovered_text(discovered: &[serde_json::Value]) {
    for id in discovered {
        println!("  {}", id);
    }
    println!("Discovered {} sources", discovered.len());
}
