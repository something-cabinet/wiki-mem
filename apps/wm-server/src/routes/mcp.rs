//! MCP over HTTP (Streamable-HTTP-shaped) endpoint.
//!
//! `POST /mcp` speaks the MCP protocol's JSON-RPC 2.0 envelope over the same
//! axum runtime that serves the web API, so the existing `x-wm-token`
//! middleware and CSRF guard protect it with zero extra wiring.
//!
//! Transport decision (spec open question): Streamable-HTTP shape, plain
//! JSON-RPC content negotiation. The request carries an `Accept` header; the
//! daemon answers with a single `application/json` JSON-RPC response when JSON
//! is acceptable (the SDK default is `application/json, text/event-stream`)
//! and with an SSE-framed `text/event-stream` message otherwise. This is the
//! stateless subset of the Streamable-HTTP spec — it needs nothing beyond axum
//! itself (the SSE framing is just `event: message\n data: <json>`), satisfies
//! the spec's "only existing axum stack" constraint, and is interoperable with
//! both the official MCP SDKs and plain HTTP clients.
//!
//! Only the tools capability is advertised; `initialize`, `notifications/*`,
//! `ping`, `tools/list` and `tools/call` are handled. Notifications (messages
//! without an `id`) receive HTTP 202 with no body, per the spec. Requests are
//! answered with JSON-RPC responses; batches are mapped element-wise.

use std::sync::Arc;

use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::{json, Value};

use wm_core::ToolRegistry;

const PROTOCOL_VERSION: &str = "2024-11-05";
const SERVER_NAME: &str = "wm-engine";
const ACCEPT_JSON: &str = "application/json";
const ACCEPT_SSE: &str = "text/event-stream";
const ACCEPT_ANY: &str = "*/*";

const CODE_PARSE_ERROR: i64 = -32700;
const CODE_INVALID_REQUEST: i64 = -32600;
const CODE_METHOD_NOT_FOUND: i64 = -32601;
const CODE_INVALID_PARAMS: i64 = -32602;

/// Handle one `POST /mcp` request: parse the JSON-RPC envelope(s), dispatch
/// tool calls through the shared registry, and render the response using the
/// client's `Accept` negotiation.
pub async fn mcp(
    State(registry): State<Arc<ToolRegistry>>,
    headers: HeaderMap,
    body: String,
) -> Response {
    let parsed: Result<Value, serde_json::Error> = serde_json::from_str(&body);
    let messages: Vec<Value> = match parsed {
        Ok(Value::Array(items)) => items,
        Ok(single @ Value::Object(_)) => vec![single],
        Ok(_) => return rpc_fault(CODE_INVALID_REQUEST, "Invalid Request"),
        Err(_) => return rpc_fault(CODE_PARSE_ERROR, "Parse error"),
    };

    let mut responses: Vec<Value> = Vec::new();
    for message in &messages {
        if let Some(response) = handle_message(&registry, message).await {
            responses.push(response);
        }
    }

    if responses.is_empty() {
        return StatusCode::ACCEPTED.into_response();
    }

    let payload = if responses.len() == 1 {
        responses.into_iter().next().unwrap_or_default()
    } else {
        Value::Array(responses)
    };

    render(payload, &headers)
}

async fn handle_message(registry: &ToolRegistry, message: &Value) -> Option<Value> {
    let id = match message.get("id") {
        Some(Value::Null) | None => return None, // notification
        Some(id) => id.clone(),
    };
    if message.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Some(rpc_error(id, CODE_INVALID_REQUEST, "Invalid Request"));
    }
    let method = match message.get("method").and_then(Value::as_str) {
        Some(method) => method,
        None => return Some(rpc_error(id, CODE_INVALID_REQUEST, "Invalid Request")),
    };
    let params = message
        .get("params")
        .cloned()
        .unwrap_or_else(|| json!({}));

    match method {
        "initialize" => Some(rpc_result(id, initialize_result(&params))),
        "tools/list" => Some(rpc_result(id, json!({ "tools": registry.list_tools() }))),
        "tools/call" => Some(call_tool(registry, id, &params).await),
        "ping" => Some(rpc_result(id, json!({}))),
        _ => Some(rpc_error(
            id,
            CODE_METHOD_NOT_FOUND,
            format!("Method not found: {method}"),
        )),
    }
}

fn initialize_result(params: &Value) -> Value {
    let requested = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or(PROTOCOL_VERSION);
    let protocol = if matches!(requested, "2024-11-05" | "2025-03-26" | "2025-06-18") {
        requested
    } else {
        PROTOCOL_VERSION
    };
    json!({
        "protocolVersion": protocol,
        "capabilities": { "tools": {} },
        "serverInfo": {
            "name": SERVER_NAME,
            "version": env!("CARGO_PKG_VERSION"),
        },
        "instructions": "Call wm_initial at the start of every session; it injects project context.",
    })
}

async fn call_tool(registry: &ToolRegistry, id: Value, params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or_default();
    if name.is_empty() {
        return rpc_error(
            id,
            CODE_INVALID_PARAMS,
            "tools/call requires a non-empty 'name'",
        );
    }
    let arguments = match params.get("arguments") {
        Some(Value::Object(map)) => Value::Object(map.clone()),
        None | Some(Value::Null) => json!({}),
        Some(_) => {
            return rpc_error(
                id,
                CODE_INVALID_PARAMS,
                "tools/call 'arguments' must be an object",
            );
        }
    };

    let result = registry.dispatch_async(name, arguments).await;
    match result {
        Ok(data) => rpc_result(
            id,
            json!({
                "content": [ { "type": "text", "text": data.to_string() } ],
                "isError": false,
            }),
        ),
        Err(e) => rpc_result(
            id,
            json!({
                "content": [ { "type": "text", "text": json!({ "error": e.message, "code": e.code }).to_string() } ],
                "isError": true,
            }),
        ),
    }
}

fn rpc_result(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn rpc_error(id: Value, code: i64, message: impl std::fmt::Display) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message.to_string() },
    })
}

fn rpc_fault(code: i64, message: &str) -> Response {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": null,
        "error": { "code": code, "message": message },
    });
    (
        StatusCode::BAD_REQUEST,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_string(&payload).unwrap_or_else(|_| "{}".into()),
    )
        .into_response()
}

fn render(payload: Value, headers: &HeaderMap) -> Response {
    let body = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".into());
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let wants_json =
        accept.is_empty() || accept.contains(ACCEPT_JSON) || accept.contains(ACCEPT_ANY);
    let wants_sse = accept.contains(ACCEPT_SSE);

    if wants_sse && !wants_json {
        let framed = format!("event: message\ndata: {body}\n\n");
        (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "text/event-stream"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            framed,
        )
            .into_response()
    } else {
        (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "application/json"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            body,
        )
            .into_response()
    }
}
