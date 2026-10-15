use crate::shared::models::SearchAction;
use crate::shared::services::call_tool;
use crate::shared::traits::Command;

const SEARCH_QUERY_TOOL: &str = "wm_search.query";
const SEARCH_RETRIEVE_TOOL: &str = "wm_search.retrieve";
const SEARCH_RESOLVE_TOOL: &str = "wm_search.resolve";
const MEMORY_TYPE_TAG: &str = " [memory]";
const DEFAULT_MODE: &str = "auto";
const UNKNOWN_ID: &str = "?";

impl Command for SearchAction {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_search_action(self).await
    }
}

async fn run_search_action(action: SearchAction) -> Result<(), anyhow::Error> {
    match action {
        SearchAction::Query {
            query,
            mode,
            r#type,
            limit,
            json,
        } => run_search_query(query, mode, r#type, limit, json).await,
        SearchAction::Retrieve {
            query,
            token_budget,
            json,
        } => run_search_retrieve(query, token_budget, json).await,
        SearchAction::Resolve { query, json } => run_search_resolve(query, json).await,
    }
}

async fn run_search_query(
    query: String,
    mode: Option<String>,
    search_type: Option<String>,
    limit: usize,
    json: bool,
) -> Result<(), anyhow::Error> {
    let args = build_query_args(&query, &mode, &search_type, limit);
    match call_tool(SEARCH_QUERY_TOOL, args).await {
        Ok(resp) => print_query_results(&resp, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn build_query_args(
    query: &str,
    mode: &Option<String>,
    search_type: &Option<String>,
    limit: usize,
) -> serde_json::Value {
    let mut args = serde_json::Map::new();
    args.insert("q".into(), serde_json::json!(query));
    if let Some(search_type) = search_type {
        args.insert("type".into(), serde_json::json!(search_type));
    }
    if let Some(mode) = mode {
        args.insert("mode".into(), serde_json::json!(mode));
    }
    args.insert("limit".into(), serde_json::json!(limit));
    serde_json::Value::Object(args)
}

fn print_query_results(resp: &serde_json::Value, json: bool) -> Result<(), anyhow::Error> {
    let mode_used = resp["mode"].as_str().unwrap_or(DEFAULT_MODE).to_string();
    let results = collect_query_results(resp);
    if json {
        return print_query_json(&mode_used, &results);
    }
    print_query_text(&mode_used, &results);
    Ok(())
}

fn collect_query_results(resp: &serde_json::Value) -> Vec<serde_json::Value> {
    resp["results"]
        .as_array()
        .map(|arr| arr.iter().map(query_result_entry).collect())
        .unwrap_or_default()
}

fn query_result_entry(result: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "score": result["score"],
        "id": result["id"],
        "type": result["type"],
    })
}

fn print_query_json(mode_used: &str, results: &[serde_json::Value]) -> Result<(), anyhow::Error> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "mode": mode_used,
            "results": results,
            "total": results.len(),
        }))?
    );
    Ok(())
}

fn print_query_text(mode_used: &str, results: &[serde_json::Value]) {
    println!("Mode: {}", mode_used);
    for result in results {
        println!(
            "  {:.2}  {}{}",
            score_of(result),
            id_of(result),
            type_tag(result)
        );
    }
    println!("{} results", results.len());
}

fn type_tag(result: &serde_json::Value) -> &'static str {
    if result["type"].as_str() == Some("memory") {
        return MEMORY_TYPE_TAG;
    }
    ""
}

async fn run_search_retrieve(
    query: String,
    token_budget: usize,
    json: bool,
) -> Result<(), anyhow::Error> {
    let payload = serde_json::json!({ "q": query, "token_budget": token_budget });
    match call_tool(SEARCH_RETRIEVE_TOOL, payload).await {
        Ok(resp) => print_retrieve(&resp, &query, token_budget, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn print_retrieve(
    resp: &serde_json::Value,
    query: &str,
    token_budget: usize,
    json: bool,
) -> Result<(), anyhow::Error> {
    if json {
        return print_retrieve_json(resp, query, token_budget);
    }
    print_retrieve_text(resp);
    Ok(())
}

fn print_retrieve_json(
    resp: &serde_json::Value,
    query: &str,
    token_budget: usize,
) -> Result<(), anyhow::Error> {
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "query": query,
            "token_budget": token_budget,
            "tokens_used": resp["tokens_used"],
            "result_count": resp["result_count"],
            "context": resp["context"],
        }))?
    );
    Ok(())
}

fn print_retrieve_text(resp: &serde_json::Value) {
    if let Some(results) = resp["results"].as_array() {
        for result in results {
            println!("  {:.2}  {}", score_of(result), id_of(result));
        }
    }
    if let Some(context) = resp["context"].as_str() {
        if !context.is_empty() {
            println!("{}", context);
        }
    }
    println!("{} context items", result_count(resp));
}

fn result_count(resp: &serde_json::Value) -> usize {
    resp["results"].as_array().map(|a| a.len()).unwrap_or(0)
}

async fn run_search_resolve(query: String, json: bool) -> Result<(), anyhow::Error> {
    let payload = serde_json::json!({ "q": query });
    match call_tool(SEARCH_RESOLVE_TOOL, payload).await {
        Ok(result) => print_resolve(&result, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn print_resolve(result: &serde_json::Value, json: bool) -> Result<(), anyhow::Error> {
    if json {
        println!("{}", serde_json::to_string_pretty(result)?);
        return Ok(());
    }
    print_resolve_text(result);
    Ok(())
}

fn print_resolve_text(result: &serde_json::Value) {
    if result["resolved"].as_bool().unwrap_or(false) {
        println!("Resolved: {} ({})", result["id"], result["title"]);
        return;
    }
    println!("Not resolved");
    print_resolve_candidates(result);
}

fn print_resolve_candidates(result: &serde_json::Value) {
    let Some(candidates) = result["candidates"].as_array() else {
        return;
    };
    for candidate in candidates {
        println!("  {:.2}  {}", candidate["score"], candidate["id"]);
    }
}

fn score_of(result: &serde_json::Value) -> f64 {
    result["score"].as_f64().unwrap_or(0.0)
}

fn id_of(result: &serde_json::Value) -> &str {
    result["id"].as_str().unwrap_or(UNKNOWN_ID)
}
