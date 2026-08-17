//! MCP-over-HTTP contract tests.
//!
//! Drives the real `wm-server` daemon's `POST /mcp` endpoint — the same axum
//! runtime that serves the web API — using the shared web token. Covers the
//! JSON-RPC envelope (initialize, tools/list, tools/call, ping, unknown
//! methods, notifications), the Streamable-HTTP `Accept` negotiation
//! (`application/json` vs `text/event-stream`), and auth rejection.

#[path = "helpers/setup.rs"]
mod setup;
use setup::setup_test_project;

#[path = "helpers/http_daemon.rs"]
mod daemon;
use daemon::DaemonHandle;

use serde_json::{json, Value};

const TOKEN_HEADER: &str = "x-wm-token";
const MCP_PATH: &str = "/mcp";

/// POST a JSON-RPC message to `/mcp` with an explicit `Accept` header and an
/// optional token (mirrors how an MCP client or curl drives the endpoint).
fn post_mcp(
    base: &str,
    body: &Value,
    token: Option<&str>,
    accept: &str,
) -> (u16, String) {
    let mut request = ureq::post(&format!("{base}{MCP_PATH}"))
        .set("content-type", "application/json")
        .set("accept", accept);
    if let Some(tok) = token {
        request = request.set(TOKEN_HEADER, tok);
    }
    match request.send_string(&body.to_string()) {
        Ok(resp) => (resp.status(), resp.into_string().unwrap_or_default()),
        Err(ureq::Error::Status(code, resp)) => (code, resp.into_string().unwrap_or_default()),
        Err(e) => (0, format!("transport error: {e}")),
    }
}

fn rpc(method: &str, id: u64, params: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params })
}

fn parse(resp: &(u16, String)) -> Value {
    assert_eq!(
        resp.0, 200,
        "expected HTTP 200, got {}: {}",
        resp.0, resp.1
    );
    serde_json::from_str(&resp.1).unwrap_or_else(|e| panic!("invalid JSON ({e}): {}", resp.1))
}

/// Write a wiki page on disk before the daemon boots, so the startup
/// `rebuild_wiki` indexes it and search is deterministic (no watcher race).
fn seed_page(root: &std::path::Path, rel: &str, frontmatter: &str, body: &str) {
    let path = root.join(".wm").join("wiki").join(rel);
    std::fs::create_dir_all(path.parent().expect("wiki page has a parent dir"))
        .expect("create wiki subdir");
    std::fs::write(&path, format!("---\n{frontmatter}---\n\n{body}\n"))
        .expect("write seeded wiki page");
}

#[test]
fn mcp_http_initialize_handshake() {
    let (_dir, root) = setup_test_project();
    let daemon = DaemonHandle::start(&root);

    let resp = parse(&post_mcp(
        &daemon.base_url,
        &rpc(
            "initialize",
            1,
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "e2e-http", "version": "1.0.0" },
            }),
        ),
        Some(&daemon.web_token),
        "application/json, text/event-stream",
    ));
    assert_eq!(resp["id"], json!(1));
    assert!(resp.get("error").is_none(), "protocol error in: {resp}");
    let result = &resp["result"];
    assert_eq!(result["protocolVersion"], json!("2024-11-05"));
    assert_eq!(result["serverInfo"]["name"], json!("wm-engine"));
    assert!(
        result["capabilities"]["tools"].is_object(),
        "tools capability must be advertised: {resp}"
    );
}

#[test]
fn mcp_http_tools_list_and_call_search() {
    let (_dir, root) = setup_test_project();
    seed_page(
        &root,
        "concepts/http-mcp.md",
        "title: HTTP MCP\ntype: concept\ntags: [mcp]\n",
        "Zirconium-http-mcp unique searchable phrase.",
    );
    let daemon = DaemonHandle::start(&root);

    let resp = parse(&post_mcp(
        &daemon.base_url,
        &rpc("tools/list", 2, json!({})),
        Some(&daemon.web_token),
        "application/json",
    ));
    let tools = resp["result"]["tools"].as_array().expect("tools array");
    assert!(
        tools.iter().any(|t| t["name"] == "wm_search.query"),
        "wm_search.query missing from tools/list: {resp}"
    );
    assert!(
        tools.iter().any(|t| t["name"] == "wm_initial"),
        "wm_initial missing from tools/list: {resp}"
    );

    let resp = parse(&post_mcp(
        &daemon.base_url,
        &rpc(
            "tools/call",
            3,
            json!({
                "name": "wm_search.query",
                "arguments": { "q": "zirconium-http-mcp", "type": "all", "limit": 10 },
            }),
        ),
        Some(&daemon.web_token),
        "application/json",
    ));
    assert_eq!(resp["id"], json!(3));
    assert!(resp.get("error").is_none(), "protocol error in: {resp}");
    let result = &resp["result"];
    assert_eq!(result["isError"], json!(false), "tool call failed: {result}");
    let text = result["content"][0]["text"]
        .as_str()
        .expect("text content block");
    let payload: Value =
        serde_json::from_str(text).expect("tool result text must embed JSON");
    let results = payload["results"].as_array().expect("results array");
    assert!(
        !results.is_empty(),
        "search should return the seeded page, got {payload}"
    );
}

#[test]
fn mcp_http_rejects_unauthenticated() {
    let (_dir, root) = setup_test_project();
    let daemon = DaemonHandle::start(&root);

    let (status, _) = post_mcp(
        &daemon.base_url,
        &rpc("tools/list", 4, json!({})),
        None,
        "application/json",
    );
    assert_eq!(status, 401, "missing token must be rejected");

    let (status, _) = post_mcp(
        &daemon.base_url,
        &rpc("tools/list", 5, json!({})),
        Some("deadbeef"),
        "application/json",
    );
    assert_eq!(status, 401, "wrong token must be rejected");
}

#[test]
fn mcp_http_streamable_sse_negotiation() {
    let (_dir, root) = setup_test_project();
    let daemon = DaemonHandle::start(&root);

    let (status, body) = post_mcp(
        &daemon.base_url,
        &rpc(
            "initialize",
            6,
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "sse-client", "version": "1.0.0" },
            }),
        ),
        Some(&daemon.web_token),
        "text/event-stream",
    );
    assert_eq!(status, 200, "SSE response should be 200: {body}");
    assert!(
        body.starts_with("event: message\n"),
        "SSE framing expected, got: {body}"
    );
    let data = body
        .strip_prefix("event: message\n")
        .and_then(|rest| rest.strip_prefix("data: "))
        .expect("data field present");
    let resp: Value = serde_json::from_str(data.trim()).expect("SSE data is JSON");
    assert_eq!(resp["id"], json!(6));
    assert!(resp.get("error").is_none(), "protocol error in: {resp}");
}

#[test]
fn mcp_http_notification_gets_202() {
    let (_dir, root) = setup_test_project();
    let daemon = DaemonHandle::start(&root);

    let (status, body) = post_mcp(
        &daemon.base_url,
        &json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
        Some(&daemon.web_token),
        "application/json",
    );
    assert_eq!(status, 202, "notifications should answer 202: {body}");
}

#[test]
fn mcp_http_unknown_method_is_json_rpc_error() {
    let (_dir, root) = setup_test_project();
    let daemon = DaemonHandle::start(&root);

    let resp = parse(&post_mcp(
        &daemon.base_url,
        &rpc("bogus/method", 7, json!({})),
        Some(&daemon.web_token),
        "application/json",
    ));
    assert_eq!(resp["id"], json!(7));
    assert_eq!(resp["error"]["code"], json!(-32601));
}

#[test]
fn mcp_http_ping() {
    let (_dir, root) = setup_test_project();
    let daemon = DaemonHandle::start(&root);

    let resp = parse(&post_mcp(
        &daemon.base_url,
        &rpc("ping", 8, json!({})),
        Some(&daemon.web_token),
        "application/json",
    ));
    assert_eq!(resp["result"], json!({}));
}
