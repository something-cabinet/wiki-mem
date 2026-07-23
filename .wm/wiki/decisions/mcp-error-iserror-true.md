---
{}
relates_to:
  - {type: references, target: wiki:tasks:mcp-direct-t2-error-mapping}
  - {type: references, target: wiki:specs:mcp-direct-handlers}
---

---
title: Decision: Handler errors use isError:true, not JSON-RPC errors
type: decision
status: approved
tags: [decision, good-call, mcp, error-handling, spec-compliance]
---

## Context

The MCP transport (`mcp_transport.rs::call_tool`) mapped every `ToolError` to `Err(ErrorData)` — a JSON-RPC protocol error. Per the MCP specification and client best practices guide, tool execution errors should be returned as `isError: true` inside a successful response, not as transport-level failures. JSON-RPC errors abort client-side scripts; `isError: true` lets the model self-correct with try/catch.

## Decision

Split error mapping in `call_tool`:
- **Dispatch-miss** (unknown tool name): JSON-RPC `ErrorData` (METHOD_NOT_FOUND)
- **Handler-returned `ToolError`**: `CallToolResult::error(...)` with `err.to_json()` as text content (`isError: true`)

## Rationale

The MCP client best practices guide is explicit: `isError: true` is the correct way to signal tool execution failures. JSON-RPC errors are for protocol-level issues. Clients that implement programmatic tool calling ("code mode") generate wrappers that convert `isError: true` into catchable exceptions — protocol errors kill the script.

## Consequences

- Added `has_tool()` method to `ToolRegistry` for dispatch-miss detection
- `mcp_transport.rs` now checks tool existence before dispatch
- All existing `ToolError` variants (NOT_FOUND, INVALID_INPUT, LOCKED, etc.) now produce proper `isError: true` responses
- Backward compatible at the MCP protocol level — clients receive the same error information, just packaged correctly

## Related

- @wiki/tasks/mcp-direct-t2-error-mapping
- @wiki/specs/mcp-direct-handlers