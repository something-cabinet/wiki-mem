---
title: error response format
id: wiki:decisions:error-response-format
type: decision
relates_to:
  - {type: implements, target: wiki:patterns:mcp-response-format}
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