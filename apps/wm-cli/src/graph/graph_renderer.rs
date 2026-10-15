use std::collections::BTreeMap;

const NO_PATH_NOTE: &str = "No path found";
const DEFAULT_KIND: &str = "page";
const MISSING_LINE: &str = "-";

pub fn print_graph_stats(resp: &serde_json::Value, json: bool) -> Result<(), anyhow::Error> {
    let nodes = resp["nodes"].as_u64().unwrap_or(0);
    let edges = resp["edges"].as_u64().unwrap_or(0);
    let type_counts = collect_type_counts(resp);
    if json {
        return print_graph_stats_json(nodes, edges, &type_counts);
    }
    print_graph_stats_text(nodes, edges, &type_counts);
    Ok(())
}

fn collect_type_counts(resp: &serde_json::Value) -> BTreeMap<String, u64> {
    let Some(types) = resp["types"].as_object() else {
        return BTreeMap::new();
    };
    types
        .iter()
        .map(|(name, count)| (name.clone(), count.as_u64().unwrap_or(0)))
        .collect()
}

fn print_graph_stats_json(
    nodes: u64,
    edges: u64,
    type_counts: &BTreeMap<String, u64>,
) -> Result<(), anyhow::Error> {
    let payload = serde_json::json!({ "nodes": nodes, "edges": edges, "types": type_counts });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn print_graph_stats_text(nodes: u64, edges: u64, type_counts: &BTreeMap<String, u64>) {
    println!("Graph stats:");
    println!("  Nodes: {}", nodes);
    println!("  Edges: {}", edges);
    print_type_counts(type_counts);
}

fn print_type_counts(type_counts: &BTreeMap<String, u64>) {
    if type_counts.is_empty() {
        return;
    }
    println!("  Types:");
    for (name, count) in type_counts {
        println!("    {}: {}", name, count);
    }
}

pub fn print_graph_path(
    resp: &serde_json::Value,
    start: &str,
    end: &str,
    json: bool,
) -> Result<(), anyhow::Error> {
    let path = resp["path"].as_array().cloned().unwrap_or_default();
    if path.is_empty() {
        return print_empty_path(start, end, json);
    }
    print_found_path(&path, json)
}

fn print_empty_path(start: &str, end: &str, json: bool) -> Result<(), anyhow::Error> {
    if json {
        let payload = serde_json::json!({ "path": [], "length": 0, "note": NO_PATH_NOTE });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }
    println!("No path found between {} and {}", start, end);
    Ok(())
}

fn print_found_path(path: &[serde_json::Value], json: bool) -> Result<(), anyhow::Error> {
    if json {
        let payload = serde_json::json!({ "path": path, "length": path.len() });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }
    print_found_path_text(path);
    Ok(())
}

fn print_found_path_text(path: &[serde_json::Value]) {
    println!("Path ({} hops):", path.len().saturating_sub(1));
    for hop in path {
        let edge = hop["edge_from_parent"].as_str().unwrap_or("");
        println!("  {}  [{}]", hop["id"], edge);
    }
}

pub fn print_subgraph(
    resp: &serde_json::Value,
    center: &str,
    depth: usize,
    json: bool,
) -> Result<(), anyhow::Error> {
    if json {
        println!("{}", serde_json::to_string_pretty(resp)?);
        return Ok(());
    }
    print_subgraph_text(resp, center, depth);
    Ok(())
}

fn print_subgraph_text(resp: &serde_json::Value, center: &str, depth: usize) {
    println!("Subgraph around {} (depth {}):", center, depth);
    print_subgraph_nodes(resp);
    let edge_count = resp["edges"].as_array().map(|list| list.len()).unwrap_or(0);
    let node_count = resp["node_count"].as_u64().unwrap_or(0);
    println!("{} nodes, {} edges", node_count, edge_count);
}

fn print_subgraph_nodes(resp: &serde_json::Value) {
    let Some(nodes) = resp["nodes"].as_array() else {
        return;
    };
    for node in nodes {
        println!("  {}  {}", node["id"], node["title"]);
    }
}

pub fn print_affected(resp: &serde_json::Value, node: &str, json: bool) -> Result<(), anyhow::Error> {
    if json {
        println!("{}", serde_json::to_string_pretty(resp)?);
        return Ok(());
    }
    print_affected_text(resp, node);
    Ok(())
}

fn print_affected_text(resp: &serde_json::Value, node: &str) {
    let affected = resp["affected"].as_array().cloned().unwrap_or_default();
    let kind = resp["kind"].as_str().unwrap_or(DEFAULT_KIND);
    println!("Affected by removing {} ({}):", node, kind);
    for entry in &affected {
        print_affected_entry(entry);
    }
    println!("{} affected", affected.len());
}

fn print_affected_entry(entry: &serde_json::Value) {
    let hops = entry["hops"].as_array().map(|list| list.len()).unwrap_or(0);
    println!("  {}  (depth {})", entry["id"].as_str().unwrap_or(""), hops);
    let Some(hops) = entry["hops"].as_array() else {
        return;
    };
    for hop in hops {
        print_hop(hop);
    }
}

fn print_hop(hop: &serde_json::Value) {
    println!(
        "      {} {} -> {} (line {}, {})",
        hop["edge_type"].as_str().unwrap_or(""),
        hop["from"].as_str().unwrap_or(""),
        hop["to"].as_str().unwrap_or(""),
        hop_line(hop),
        hop["provenance"].as_str().unwrap_or(""),
    );
}

fn hop_line(hop: &serde_json::Value) -> String {
    hop["line"]
        .as_u64()
        .map(|line| line.to_string())
        .unwrap_or_else(|| MISSING_LINE.to_string())
}

pub fn print_neighbors(resp: &serde_json::Value, json: bool) -> Result<(), anyhow::Error> {
    let neighbors = resp["neighbors"].as_array().cloned().unwrap_or_default();
    if json {
        let payload = serde_json::json!({ "neighbors": neighbors, "total": neighbors.len() });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }
    for neighbor in &neighbors {
        println!("  {}  {}", neighbor["id"], neighbor["title"]);
    }
    println!("{} neighbors", neighbors.len());
    Ok(())
}
