---
id: wiki:decisions:error-response-format
title: "Decision: Flat MCP Error Objects"
type: decision
tags: [mcp, error-handling, json-rpc]
status: reviewed
confidence: high
decision:
  context: |
    ToolError::to_json() was wrapping error details in {"error": {"code": ..., "message": ...}}. The MCP transport layer (JsonRpcResponse::error) then wrapped this AGAIN in the JSON-RPC error envelope, producing a double-wrapped response.
  options:
    - "Fix to_json() to return flat code+message object"
    - "Keep double-wrap and fix transport to strip outer wrapper"
  rationale: |
    The simpler fix is to make to_json() return what the transport expects — a plain code+message object. This avoids nested error objects and matches the JSON-RPC spec where the transport layer owns the outer "error" envelope.
  outcome: |
    ToolError::to_json() now returns {"code": "REQUIRED_FIELD", "message": "..."}. MCP clients now receive correctly formatted JSON-RPC error responses. MCP E2E tests validate this format.
relates_to:
  - {type: implements, target: wiki:patterns:mcp-response-format}
  - {type: references, target: wiki:tasks:task-s2ff4x-mcp-e2e-integration-tests}
---
id: wiki:decisions:error-response-format

## Context

`ToolError::to_json()` was wrapping error details in `{"error": {"code": ..., "message": ...}}`. The MCP transport layer (`JsonRpcResponse::error`) then wrapped this AGAIN in the JSON-RPC error envelope, producing `{"error": {"error": {"code": ..., "message": ...}}}`.

## Chosen approach

`ToolError::to_json()` now returns the error object directly: `{"code": "REQUIRED_FIELD", "message": "id is required"}`. The transport layer adds the outer `"error"` wrapper.

## Why not the other way

Alternative was to keep the double-wrap and change the transport to strip the outer wrapper. But the simpler fix is to make `to_json()` return what the transport expects — a plain code+message object.

## Outcome

MCP clients now receive correctly formatted JSON-RPC error responses. MCP E2E tests validate this format.

## Source

@wiki/tasks/s2ff4x
