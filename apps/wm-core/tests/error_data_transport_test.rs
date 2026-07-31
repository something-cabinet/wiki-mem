#[path = "helpers/mcp.rs"]
mod helpers;
use helpers::MCPClient;

#[path = "helpers/setup.rs"]
mod setup;

fn setup_mcp_test() -> (tempfile::TempDir, MCPClient) {
    let (dir, root) = setup::setup_test_project();
    let client = MCPClient::start(&root);
    (dir, client)
}

const JSON_RPC_INVALID_PARAMS: i64 = -32602;
const JSON_RPC_INTERNAL_ERROR: i64 = -32603;

fn expect_field<'a>(value: &'a serde_json::Value, key: &str) -> &'a serde_json::Value {
    value
        .get(key)
        .unwrap_or_else(|| panic!("expected {} in response, got: {}", key, value))
}

#[test]
fn test_error_data_structured_on_not_found() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let resp = client
        .send_request_raw(
            "tools/call",
            serde_json::json!({
                "name": "wm_page",
                "arguments": { "action": "get", "id": "nonexistent:id" },
            }),
        )
        .expect("tools/call failed");

    let error = expect_field(&resp, "error");
    assert_eq!(
        error.get("code").and_then(|v| v.as_i64()),
        Some(JSON_RPC_INVALID_PARAMS),
        "expected INVALID_PARAMS JSON-RPC code, got: {}",
        error
    );

    let data = expect_field(error, "data");
    assert_eq!(data.get("code").and_then(|v| v.as_str()), Some("NOT_FOUND"));
    let message = data.get("message").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        message.contains("not found"),
        "expected message to mention 'not found', got: {}",
        message
    );
    assert!(
        data.get("hint").is_some(),
        "expected hint in error.data, got: {}",
        data
    );

    let err = client
        .call_tool(
            "wm_page",
            serde_json::json!({ "action": "get", "id": "nonexistent:id" }),
        )
        .unwrap_err();
    assert!(
        err.contains("not found"),
        "expected error message to mention 'not found', got: {}",
        err
    );
}

#[test]
fn test_error_data_structured_on_invalid_input() {
    let (_dir, mut client) = setup_mcp_test();
    client.initialize().expect("initialize");

    let tools = client.list_tools().expect("list_tools");
    assert!(
        tools.contains(&"wm_page".to_string()),
        "expected wm_page tool to be registered"
    );

    let resp = client
        .send_request_raw(
            "tools/call",
            serde_json::json!({
                "name": "wm_page",
                "arguments": { "action": "fly" },
            }),
        )
        .expect("tools/call failed");

    let error = expect_field(&resp, "error");
    assert_eq!(
        error.get("code").and_then(|v| v.as_i64()),
        Some(JSON_RPC_INTERNAL_ERROR),
        "expected INTERNAL_ERROR JSON-RPC code, got: {}",
        error
    );

    let data = expect_field(error, "data");
    assert_eq!(
        data.get("code").and_then(|v| v.as_str()),
        Some("SERDE_ERROR")
    );
    let message = data.get("message").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        message.contains("fly"),
        "expected message to mention the invalid action, got: {}",
        message
    );
}
