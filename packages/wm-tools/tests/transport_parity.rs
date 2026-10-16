#[path = "helpers/mcp.rs"]
mod helpers;
use helpers::MCPClient;

#[path = "helpers/inproc.rs"]
mod inproc;
use inproc::{call_err, call_ok, setup_in_process};

use serde_json::json;

const PARITY_PAGE: &str = "---\ntitle: Transport Parity Page\ntype: concept\nid: wiki:concepts:transport-parity\nstatus: draft\ntags: [parity]\n---\n\n## Overview\n\nThe quasitransportmarker phrase appears only on this page so the query below resolves to exactly one result.\n";

const MARKER_QUERY: &str = "quasitransportmarker";
const SEARCH_TOOL: &str = "wm_search.query";
const REBUILD_TOOL: &str = "wm_index_rebuild";
const RESULT_LIMIT: usize = 10;

#[tokio::test(flavor = "multi_thread")]
async fn both_transports_return_identical_search_results() {
    let ((_dir, root, _engine, registry), _cwd) = setup_in_process().await;
    std::fs::write(
        root.join(".wm/wiki/concepts/transport-parity.md"),
        PARITY_PAGE,
    )
    .expect("write parity page");

    let mut client = MCPClient::start(&root);

    let stdio_tools = client.list_tools().expect("stdio list_tools");
    assert!(
        stdio_tools.contains(&SEARCH_TOOL.to_string()),
        "stdio transport must expose {SEARCH_TOOL}, got: {stdio_tools:?}"
    );

    let rebuild = json!({ "skip_embed": true });
    client
        .call_tool(REBUILD_TOOL, rebuild.clone())
        .expect("stdio rebuild");
    call_ok(&registry, REBUILD_TOOL, rebuild).await;

    let query = json!({
        "q": MARKER_QUERY,
        "mode": "keyword",
        "limit": RESULT_LIMIT,
        "recency": false,
    });

    let stdio = client
        .call_tool(SEARCH_TOOL, query.clone())
        .expect("stdio search");
    let in_process = call_ok(&registry, SEARCH_TOOL, query).await;

    let stdio_results = stdio
        .get("results")
        .cloned()
        .unwrap_or(serde_json::json!(null));
    let in_process_results = in_process
        .get("results")
        .cloned()
        .unwrap_or(serde_json::json!(null));

    assert!(
        in_process_results
            .as_array()
            .map(|results| !results.is_empty())
            .unwrap_or(false),
        "query must match the seeded page, got: {in_process}"
    );
    assert_eq!(
        stdio_results, in_process_results,
        "stdio and in-process transports must return identical results\nstdio: {stdio}\nin-process: {in_process}"
    );

    assert!(
        client.call_tool(SEARCH_TOOL, json!({})).is_err(),
        "stdio transport must reject a query without q"
    );
    let in_process_err = call_err(&registry, SEARCH_TOOL, json!({})).await;
    assert!(
        !in_process_err.message.is_empty(),
        "in-process transport must reject a query without q"
    );
}
