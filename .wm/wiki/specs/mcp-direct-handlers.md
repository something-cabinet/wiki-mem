---
id: wiki:specs:mcp-direct-handlers
title: MCP direct handler registration
type: spec
status: approved
tags: [spec, mcp, refactor, engine, approved]
---
id: wiki:specs:mcp-direct-handlers

## Overview

`wm-cli mcp` currently registers proxy handlers that route every tool call to `wm-server` via HTTP. Replace this with direct in-process handler registration using `wm_core::mcp::tools::register_all_tools()`, matching the pattern used by Knowns (`internal/mcp/server.go` — create engine, register handlers, serve stdio). This eliminates the network hop, removes a hardcoded tool list, and makes the MCP server self-contained.

The refactor also corrects the externally visible tool catalog: the proxy's `STATIC_TOOLS` list is stale (~26 of 50 advertised names no longer exist in the engine registry; ~25 registered tools are unreachable) and serves empty descriptions/input schemas, so `tools/list` after this change is intentionally *not* parity with the old proxy — parity is defined against wm-server, which uses the same `register_all_tools()` source.

## Locked Decisions

- D1: `wm-cli mcp` creates EngineState in-process and registers real handlers directly via `register_all_tools()`
- D2: EngineState is full read-write (create/update tasks, pages, memory, etc.)
- D3: `mcp_proxy.rs` is deleted entirely — no proxy fallback retained
- D4: `wm-cli serve` removed. `wm-cli web` starts the HTTP server (axum) only. MCP is started independently by `mcpmon` via `opencode.json` config — no lifecycle coupling

## Requirements

### Functional Requirements

- FR-1: `wm-cli mcp` creates EngineState from the project root and registers tool handlers in-process
- FR-2: `register_all_tools()` is called on the registry with the in-process engine (same as wm-server does)
- FR-3: `mcp_proxy.rs` is deleted; `STATIC_TOOLS` and all proxy handler code removed
- FR-4: `wm-cli serve` command removed (aliased or replaced by `wm-cli mcp`)
- FR-5: `wm-cli web` starts the HTTP server (axum on configurable port). MCP is a separate process managed by `mcpmon` — no lifecycle coupling
- FR-6: Writes to wiki files from MCP (task create, page update) are visible to other processes via existing mtime staleness detection — no extra sync needed
- FR-7: `McpServer::call_tool` error mapping is split: dispatch-miss → JSON-RPC `ErrorData`; handler-returned `ToolError` → `CallToolResult::error(...)` with `err.to_json()` as text content (`isError: true`)

### Non-Functional Requirements

- NFR-1: MCP server starts within 500ms of command invocation (cold start with EngineState initialization)
- NFR-2: No network calls on MCP tool execution path
- NFR-3: Existing `opencode.json` MCP config (`mcpmon ./target/debug/wm-cli mcp`) continues to work unchanged
- NFR-4: Tool **execution** errors (any `ToolError` returned by a matched handler) MUST be returned as a successful `tools/call` response with `isError: true` and a text content block containing the `ToolError` JSON (`code`, `message`, `hint`). Only protocol-level failures (unknown tool name at dispatch, malformed request) may surface as JSON-RPC errors. Rationale: MCP clients treat `isError: true` as a model-recoverable result (code-mode wrappers turn it into catchable exceptions); transport/JSON-RPC errors abort client-side scripts. NOTE: the current `mcp_transport.rs::call_tool` maps every `ToolError` to `Err(ErrorData)` — a JSON-RPC error — so this is a required code change, not inherited behavior.
- NFR-5: Declared capabilities must match actual behavior. The tool set is static for the process lifetime, so the server MUST NOT advertise `tools.listChanged` and MUST NOT emit `notifications/tools/list_changed` (clients re-index cached tool catalogs on that notification). If list-time tool filtering is ever added (e.g., by permission preset), emitting `list_changed` becomes mandatory.
- NFR-6: `tools/list` from `wm-cli mcp` MUST be identical to wm-server's (names, descriptions, input schemas — same `register_all_tools()` source). Every tool MUST have a non-empty description and a real input schema; consolidated action-enum tools MUST enumerate their actions in the description (client discovery indexes name + description only).

## Acceptance Criteria

- [ ] AC-1: `wm-cli mcp` starts without wm-server running and tools respond correctly
- [ ] AC-2: MCP `tools/list` returns all registered tools, each with a non-empty description and a non-empty JSON input schema (no `{"type":"object","properties":{}}` placeholders)
- [ ] AC-3: MCP `tools/call` for create/update operations persists to wiki files and is visible on next read
- [ ] AC-4: `mcp_proxy.rs` and its `STATIC_TOOLS` constant no longer exist in the codebase
- [ ] AC-5: `wm-cli serve` no longer exists as a command
- [ ] AC-6: `wm-cli web` starts the HTTP server only; it does not spawn or embed an MCP process (per D4 — no lifecycle coupling)
- [ ] AC-7: `cargo check -p wm-cli -p wm-core -p wm-server` passes clean
- [ ] AC-8: `wm setup opencode` generates correct config referencing `wm-cli mcp`
- [ ] AC-9: Error semantics — calling `wm_page` (get) with a nonexistent id returns a successful `tools/call` response with `isError: true` and content containing `"code": "NOT_FOUND"`; calling an unregistered tool name returns a JSON-RPC error
- [ ] AC-10: `tools/list` from `wm-cli mcp` matches `tools/list` from wm-server exactly (names, descriptions, inputSchemas)
- [ ] AC-11: The `initialize` response advertises the tools capability without `listChanged`, and no `list_changed` notification is ever sent

## Scenarios

### Scenario 1: Normal MCP startup
**Given** a project with `.wm/config.json` in the current or parent directory
**When** `wm-cli mcp` is started
**Then** it creates an EngineState from the project root
**And** registers all tool handlers directly
**And** serves stdio MCP transport

### Scenario 2: MCP without wm-server
**Given** wm-server is not running
**When** `wm-cli mcp` is started
**Then** it works independently — no connection refused errors, no retries

### Scenario 3: Cross-process visibility
**Given** wm-server (HTTP) is running and wm-cli mcp is also running
**When** a task is created via `wm-cli mcp`
**Then** wm-server (HTTP) sees the new task on next read (via mtime staleness detection)

## Technical Notes

- The `Commands::Mcp` handler in `apps/wm-cli/src/main.rs` currently calls `mcp_proxy::register_proxy_handlers()`. Replace with `wm_core::mcp::tools::register_all_tools()` using an EngineState created via `MainEngine::with_root()` or equivalent.
- The `EngineState` constructor already accepts `(config, project_root)` and can be created in wm-cli directly since wm-cli depends on wm-core.
- `mcp_proxy.rs` (162 lines) and its `STATIC_TOOLS` list (50 entries) are fully deleted.
- **Verified catalog drift:** ~26 of the 50 `STATIC_TOOLS` names (`wm_page.*`, `wm_task.*`, `wm_memory.*`, `wm_time.*`, `wm_source.*`, `wm_template.*`, bare `wm_version`) no longer exist in the engine registry; ~25 registered tools (consolidated action tools, `wm_project.*`, `wm_skill.trigger`, `wm_ref.*`, `wm_log.*`, `wm_code.*`, `wm_lint.fix`, `wm_version.*`) are unreachable via the proxy. This is a client-visible breaking catalog change — ship with a version bump (clients cache tool defs keyed by server name/version) and follow up with a docs sweep.
- **Enabled by this refactor (follow-ups, out of scope):**
  - *outputSchema / structuredContent:* handlers already return typed outputs (e.g., `WmPageListOutput`); `register_typed` can derive `outputSchema` from the return type in one place. MCP clients generating typed stubs ("code mode") depend on outputSchema; the proxy made this impossible without cross-binary schema duplication.
  - *Naming convention:* dotted names (`wm_search.query`) fall outside some clients' function-name alphabets (hosts sanitize). Consolidated-vs-dotted and dot-vs-underscore is a deliberate follow-up decision.
- **Error mapping detail:** `mcp_transport.rs:74-76` currently maps every `ToolError` to `Err(ErrorData)` — a JSON-RPC protocol error. After this refactor, `call_tool` must split: dispatch-miss (unknown tool) → JSON-RPC `ErrorData`; handler-returned `ToolError` → `CallToolResult::error(...)` with `err.to_json()` as text content (`isError: true`).

## Open Questions

None resolved — all decisions locked.
