use crate::shared::models::DecideArgs;
use crate::shared::services::call_tool;
use crate::shared::traits::Command;

const DECISION_TOOL: &str = "wm_decision";
const ACTION_ANSWER: &str = "answer";

impl Command for DecideArgs {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_decision(self).await
    }
}

async fn run_decision(args: DecideArgs) -> Result<(), anyhow::Error> {
    let payload = build_decision_payload(&args);
    match call_tool(DECISION_TOOL, payload).await {
        Ok(result) => print_decision(&result, args.json)?,
        Err(error) => eprintln!("Error: {error}"),
    }
    Ok(())
}

fn build_decision_payload(args: &DecideArgs) -> serde_json::Value {
    let mut payload = serde_json::Map::new();
    payload.insert("action".to_owned(), serde_json::json!(ACTION_ANSWER));
    insert_optional(&mut payload, "page", &args.page);
    insert_optional(&mut payload, "state", &args.state);
    insert_optional(&mut payload, "type", &args.page_type);
    serde_json::Value::Object(payload)
}

fn insert_optional(
    payload: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: &Option<String>,
) {
    if let Some(value) = value {
        payload.insert(key.to_owned(), serde_json::json!(value));
    }
}

fn print_decision(result: &serde_json::Value, json: bool) -> Result<(), anyhow::Error> {
    if json {
        println!("{}", serde_json::to_string_pretty(result)?);
        return Ok(());
    }
    print_decision_text(result);
    Ok(())
}

fn print_decision_text(result: &serde_json::Value) {
    println!(
        "model: {}  chunks: {}  truncated: {}",
        result["model"], result["chunks"], result["truncated"]
    );
    let Some(answers) = result["answers"].as_object() else {
        return;
    };
    for (id, answer) in answers {
        println!("  {id}: {} (p={})", answer["value"], answer["probability"]);
    }
}
