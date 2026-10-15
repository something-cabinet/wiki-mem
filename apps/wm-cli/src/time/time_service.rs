use crate::time::TimeAction;
use crate::shared::services::call_tool;
use crate::shared::traits::Command;

const TIME_TOOL: &str = "wm_time";

impl Command for TimeAction {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_time_action(self).await
    }
}

async fn run_time_action(action: TimeAction) -> Result<(), anyhow::Error> {
    match action {
        TimeAction::Start { id, .. } => start_time(&id).await,
        TimeAction::Stop { id, .. } => stop_time(&id).await,
        TimeAction::Add { id, duration, .. } => add_time(&id, &duration).await,
        TimeAction::Report { json } => report_time(json).await,
    }
}

async fn start_time(id: &str) -> Result<(), anyhow::Error> {
    let payload = serde_json::json!({ "action": "start", "id": id });
    match call_tool(TIME_TOOL, payload).await {
        Ok(response) => print_time_started(&response, id),
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn print_time_started(response: &serde_json::Value, id: &str) {
    let now = response["time_started"].as_str().unwrap_or("");
    println!("Time tracking started for {} at {}", id, now);
}

async fn stop_time(id: &str) -> Result<(), anyhow::Error> {
    let payload = serde_json::json!({ "action": "stop", "id": id });
    match call_tool(TIME_TOOL, payload).await {
        Ok(response) => print_time_stopped(&response, id),
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn print_time_stopped(response: &serde_json::Value, id: &str) {
    let elapsed = response["time_spent"].as_str().unwrap_or("0h 0m");
    println!("Time stopped for {}: {}", id, elapsed);
}

async fn add_time(id: &str, duration: &str) -> Result<(), anyhow::Error> {
    let payload = serde_json::json!({ "action": "add", "id": id, "duration": duration });
    match call_tool(TIME_TOOL, payload).await {
        Ok(_) => println!("Time added for {}: {}", id, duration),
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

async fn report_time(json: bool) -> Result<(), anyhow::Error> {
    let payload = serde_json::json!({ "action": "report" });
    match call_tool(TIME_TOOL, payload).await {
        Ok(response) => print_time_report(&response, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn print_time_report(response: &serde_json::Value, json: bool) -> Result<(), anyhow::Error> {
    let tasks = response["tasks"].as_array().cloned().unwrap_or_default();
    let total_hours = response["total_hours"].as_f64().unwrap_or(0.0);
    if json {
        return print_time_report_json(&tasks, total_hours);
    }
    print_time_report_text(&tasks, total_hours);
    Ok(())
}

fn print_time_report_json(
    tasks: &[serde_json::Value],
    total_hours: f64,
) -> Result<(), anyhow::Error> {
    let payload = serde_json::json!({ "tasks": tasks, "total_hours": total_hours });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn print_time_report_text(tasks: &[serde_json::Value], total_hours: f64) {
    println!("Time report:");
    for task in tasks {
        println!("  {}  {}", task["time_spent"], task["id"]);
    }
    println!("Total: {:.1}h", total_hours);
}
