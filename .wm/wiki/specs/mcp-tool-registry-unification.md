---
id: wiki:specs:mcp-tool-registry-unification
title: MCP Tool Registry Unification
type: spec
---
id: wiki:specs:mcp-tool-registry-unification

## Overview

Remove the HTTP proxy layer from `wm-cli mcp` so MCP tool handlers call the engine directly instead of forwarding through an embedded HTTP server. Refactor `wm-server` to accept a `ToolRegistry` externally instead of via a `OnceLock` global, eliminating the double registration of all tool handlers.

Currently, every MCP tool call does: `rmcp → ureq POST → axum dispatch → engine`. The axum HTTP server and the MCP server each maintain independent `ToolRegistry` instances, both populated by the same `register_all_tools()` call. This is a leftover from when `wm-mcp` and `wm-server` were separate binaries that couldn't share memory.

## Locked Decisions

- D1: `wm-cli mcp` does NOT start the background HTTP server. Graph rebuild happens on startup and via `wm_index.rebuild` on-demand.
- D2: `wm-cli serve` still works independently — it creates its own `ToolRegistry` for the web UI.
- D3: `wm-server` is refactored to accept an `Arc<ToolRegistry>` as a parameter instead of using a `OnceLock` global.

## Requirements

### Functional Requirements

- FR-1: `wm-cli mcp` must register and serve tool handlers directly via rmcp stdio, with no HTTP hop.
- FR-2: The tool handler set in MCP mode must be identical to the current set (all 19 domain modules).
- FR-3: `wm-cli serve` must continue to serve the Angular web UI and HTTP API routes with full tool dispatch.
- FR-4: `wm-server` must accept a `ToolRegistry` from the caller, not initialize one internally via `OnceLock`.
- FR-5: `wm-server` must provide a convenience constructor that creates its own `ToolRegistry` when no external one is provided.
- FR-6: The `ureq` dependency must be removed from `wm-cli/Cargo.toml`.

### Non-Functional Requirements

- NFR-1: No duplicate handler closure registration when MCP and HTTP share the same process.
- NFR-2: Zero memory overhead for MCP-only mode — no axum server, no HTTP listener, no rebuild loop.
- NFR-3: The socket/serve/run functions in `wm-server` must remain functionally identical (same routes, same behavior).
- NFR-4: It must be possible to add MCP+HTTP coexistence later without further refactoring (single `Arc<ToolRegistry>` shared by both).

## Acceptance Criteria

- [ ] AC-1: `wm-cli mcp` starts and serves all tools via rmcp stdio without starting any HTTP server.
- [ ] AC-2: All 19+ tool domains are callable from OpenCode/Claude with correct inputs, outputs, and error propagation.
- [ ] AC-3: `wm-cli serve` starts the HTTP server with all routes (web UI, `/api/tools`, `/api/search`, etc.) working correctly.
- [ ] AC-4: `wm-core` crate has `impl ServerHandler for Arc<ToolRegistry>` delegating to the inner registry.
- [ ] AC-5: `wm-server::build_api_router_with` accepts `Arc<ToolRegistry>` instead of pulling from `OnceLock`.
- [ ] AC-6: `wm-server::build_api_router` creates its own `ToolRegistry` internally as a convenience wrapper.
- [ ] AC-7: `wm-cli` Cargo.toml no longer depends on `ureq`.
- [ ] AC-8: No regressions in existing `wm-cli serve` and `wm-cli <cmd>` (search, page, task, etc.) commands.
- [ ] AC-9: Both `cargo build -p wm-cli` and `cargo build -p wm-cli --no-default-features` compile successfully.

## Scenarios

### Scenario 1: MCP Server Startup
**Given** a project with a `.wm` wiki
**When** the user runs `wm-cli mcp`
**Then** the engine loads wiki files and rebuilds the graph
**And** a `ToolRegistry` is created with all tool handlers registered
**And** the rmcp stdio server starts (no HTTP listener)
**And** OpenCode/Claude can list and call all tools

### Scenario 2: Tool Execution
**Given** `wm-cli mcp` is running
**When** OpenCode calls `wm_search.query`
**Then** the rmcp handler dispatches directly to the search handler in `wm-core`
**And** the result is returned via rmcp response
**And** no HTTP connection is made

### Scenario 3: HTTP Server Independence
**Given** no MCP server is running
**When** the user runs `wm-cli serve --port 8080`
**Then** the HTTP server starts on port 8080
**And** all API routes (`/api/tools`, `/api/search`, `/api/pages/*`, etc.) work
**And** the Angular web UI is served

### Scenario 4: On-Demand Rebuild
**Given** `wm-cli mcp` is running
**When** wiki files are modified externally
**Then** the engine's `stale_flag` is set (by file-writing tools)
**And** the graph is rebuilt on next `wm_index.rebuild` call
**And** no background loop runs

## Technical Notes

### Code Changes

#### `apps/wm-core/src/mcp/transport.rs`
Add `impl ServerHandler for Arc<ToolRegistry>` delegating all trait methods to `**self`.

#### `apps/wm-server/src/lib.rs`
- Remove `OnceLock<Arc<ToolRegistry>>`
- Add `registry: Arc<ToolRegistry>` to `AppState`
- `handle_tool_call` reads from `state.registry`
- `build_api_router_with(engine, registry, web_dist)` accepts registry as parameter
- `build_api_router(engine)` creates its own ToolRegistry as convenience wrapper

#### `apps/wm-cli/src/main.rs`
- `wm-cli mcp`: create engine, create ToolRegistry, register tools directly, serve MCP
- Remove `start_wm_server_background` call
- Remove `ureq`-based proxy handler registration

#### `apps/wm-cli/Cargo.toml`
- Remove `ureq = { version = "2.10", features = ["json"] }`

### Backward Compatibility
- `build_api_router(engine)` is unchanged
- `start_background_server(engine)` uses `build_api_router` internally
- `run_server(engine, port)` uses `build_api_router` internally
