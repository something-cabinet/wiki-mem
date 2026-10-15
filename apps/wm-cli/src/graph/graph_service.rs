use crate::graph::GraphAction;
use crate::shared::services::call_tool;
use crate::shared::traits::Command;

use super::graph_export_renderer::graph_export;
use super::graph_renderer::{
    print_affected, print_graph_path, print_graph_stats, print_neighbors, print_subgraph,
};

const GRAPH_STATS_TOOL: &str = "wm_graph.stats";
const GRAPH_PATH_TOOL: &str = "wm_graph.path";
const GRAPH_SUBGRAPH_TOOL: &str = "wm_graph.subgraph";
const GRAPH_AFFECTED_TOOL: &str = "wm_graph.affected";
const GRAPH_NEIGHBORS_TOOL: &str = "wm_graph.neighbors";
const DEFAULT_SUBGRAPH_DEPTH: usize = 1;
const MAX_SUBGRAPH_DEPTH: usize = 5;

impl Command for GraphAction {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_graph_action(self).await
    }
}

async fn run_graph_action(action: GraphAction) -> Result<(), anyhow::Error> {
    match action {
        GraphAction::Stats { json } => graph_stats(json).await,
        GraphAction::Path {
            start,
            end,
            max_depth,
            json,
        } => graph_path(&start, &end, max_depth, json).await,
        GraphAction::Subgraph { center, depth, json } => graph_subgraph(&center, depth, json).await,
        GraphAction::Affected {
            node,
            max_depth,
            json,
        } => graph_affected(&node, max_depth, json).await,
        GraphAction::Neighbors { id, query, json } => graph_neighbors(&id, query, json).await,
        GraphAction::Export { format, out } => graph_export(format, out),
    }
}

async fn graph_stats(json: bool) -> Result<(), anyhow::Error> {
    match call_tool(GRAPH_STATS_TOOL, serde_json::json!({})).await {
        Ok(resp) => print_graph_stats(&resp, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

async fn graph_path(
    start: &str,
    end: &str,
    max_depth: Option<usize>,
    json: bool,
) -> Result<(), anyhow::Error> {
    let args = build_path_args(start, end, max_depth);
    match call_tool(GRAPH_PATH_TOOL, args).await {
        Ok(resp) => print_graph_path(&resp, start, end, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn build_path_args(start: &str, end: &str, max_depth: Option<usize>) -> serde_json::Value {
    let mut args = serde_json::Map::new();
    args.insert("start".into(), serde_json::json!(start));
    args.insert("end".into(), serde_json::json!(end));
    if let Some(depth) = max_depth {
        args.insert("max_depth".into(), serde_json::json!(depth));
    }
    serde_json::Value::Object(args)
}

async fn graph_subgraph(
    center: &str,
    depth: Option<usize>,
    json: bool,
) -> Result<(), anyhow::Error> {
    let resolved_depth = depth
        .unwrap_or(DEFAULT_SUBGRAPH_DEPTH)
        .min(MAX_SUBGRAPH_DEPTH);
    let args = serde_json::json!({ "center": center, "depth": resolved_depth });
    match call_tool(GRAPH_SUBGRAPH_TOOL, args).await {
        Ok(resp) => print_subgraph(&resp, center, resolved_depth, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

async fn graph_affected(
    node: &str,
    max_depth: Option<usize>,
    json: bool,
) -> Result<(), anyhow::Error> {
    let args = build_affected_args(node, max_depth);
    match call_tool(GRAPH_AFFECTED_TOOL, args).await {
        Ok(resp) => print_affected(&resp, node, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn build_affected_args(node: &str, max_depth: Option<usize>) -> serde_json::Value {
    let mut args = serde_json::Map::new();
    args.insert("node".into(), serde_json::json!(node));
    if let Some(depth) = max_depth {
        args.insert("max_depth".into(), serde_json::json!(depth));
    }
    serde_json::Value::Object(args)
}

async fn graph_neighbors(
    id: &str,
    query: Option<String>,
    json: bool,
) -> Result<(), anyhow::Error> {
    let args = build_neighbors_args(id, query);
    match call_tool(GRAPH_NEIGHBORS_TOOL, args).await {
        Ok(resp) => print_neighbors(&resp, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn build_neighbors_args(id: &str, query: Option<String>) -> serde_json::Value {
    let mut args = serde_json::Map::new();
    args.insert("id".into(), serde_json::json!(id));
    if let Some(query) = query {
        args.insert("query".into(), serde_json::json!(query));
    }
    serde_json::Value::Object(args)
}
