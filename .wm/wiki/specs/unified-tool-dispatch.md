---
title: Unified Tool Dispatch — Single Source of Truth for Web UI + MCP
type: spec
status: draft
tags: [spec, architecture, tauri, mcp, dispatch]
---

## Overview

The Web UI (Angular → Tauri IPC) and AI agents (MCP protocol) both call the same backend engine (`EngineState`), but through entirely different dispatch mechanisms:

- **MCP**: `ToolRegistry` → registered domain handlers → `EngineState` (unified, in `wm_core::mcp::tools`)
- **Web UI**: 15 individual `#[tauri::command]` functions → `EngineState` directly (hand-written, in `apps/wm-web/src-tauri/src/commands.rs`)

This creates duplication, drift risk, and extra maintenance. Every new operation needs both an MCP tool handler AND a Tauri command. This spec defines a single dispatch layer that both MCP and Tauri use.

## Locked Decisions

- D1: Tauri commands **route through the same `ToolRegistry`** that MCP uses. One handler per operation, one registration point.
- D2: The Angular frontend **does not change** — it still calls `invoke()` with the same payloads. Only the Rust backend dispatch changes.
- D3: The `ToolRegistry` stays in `wm_core::mcp::transport` — no new crate. Tauri imports it.

## Requirements

### Functional Requirements

- FR-1: A `dispatch_tool(name: &str, params: Value) -> Result<Value, String>` function that Tauri commands call, backed by the same `ToolRegistry` that MCP uses
- FR-2: All existing Tauri commands (`get_initial`, `search`, page CRUD, task board, graph, memory, etc.) are replaced with a generic `invoke_tool` command taking tool name + JSON params
- FR-3: The Angular frontend's `invoke()` calls continue to work with the same payload shapes (backward compatible)
- FR-4: MCP continues to work identically — same handlers, same `ToolRegistry` registration
- FR-5: Error responses from `dispatch_tool` follow a consistent format (same as MCP's ToolError → JSON)
- FR-6: The `ToolRegistry` is shareable between MCP transport and Tauri (already `Arc`-based)

### Non-Functional Requirements

- NFR-1: Zero behavioral change for Angular frontend — all existing views render identically
- NFR-2: All existing E2E journeys pass unchanged
- NFR-3: No new Rust dependencies
- NFR-4: Build time must not increase significantly (no new heavy crates)

## Acceptance Criteria

- [ ] AC-1: A `dispatch_tool` function exists in `wm_core::mcp::transport` (or re-exported) that takes tool name + JSON params and returns a result
- [ ] AC-2: `commands.rs` has ONE generic `invoke_tool(name, params)` Tauri command instead of 15 individual handlers
- [ ] AC-3: Angular `invoke('invoke_tool', { name: 'wm_search.query', params: { q: 'test' } })` returns same results as old `invoke('search', { payload: { q: 'test' } })` — or via a mapping layer
- [ ] AC-4: MCP `tools/list` and `tools/call` continue to return all tools correctly with their schemas
- [ ] AC-5: All 14 E2E journeys pass (or equivalently, all views render with correct data)
- [ ] AC-6: The Tauri `State<Arc<EngineState>>` is also used to access the `ToolRegistry` (or the registry is constructed from `EngineState`)

## Scenarios

### Scenario 1: Angular Calls Search
**Given** the user types a query in the search view
**When** the Angular component calls `invoke('invoke_tool', { name: 'wm_search.query', params: { q: 'test' } })`
**Then** the Tauri command extracts the name, looks up the handler in `ToolRegistry`, calls it with the params
**And** returns the same JSON structure the frontend expects

### Scenario 2: AI Agent Calls Same Search
**Given** an AI agent sends `tools/call` with `{ name: "wm_search.query", arguments: { q: "test" } }`
**When** the MCP server receives it
**Then** it dispatches through the same `ToolRegistry` handler
**And** returns the same results in MCP JSON-RPC format

### Scenario 3: New Tool Added
**Given** a developer adds a new operation
**When** they register one handler in `ToolRegistry`
**Then** both MCP and Tauri can call it immediately — no separate Tauri command needed

## Technical Notes

### Current Tauri dispatch (before)
```rust
// commands.rs — 15 individual commands
#[tauri::command]
pub fn search(state: State<'_, Arc<EngineState>>, payload: SearchPayload) -> Result<Value, String> {
    // custom logic, hand-written error handling
    search::query::run_unified_search(&state, &qp)...
}

#[tauri::command]
pub fn get_initial(state: State<'_, Arc<EngineState>>) -> Result<Value, String> {
    // different custom logic
}
```

### Target Tauri dispatch (after)
```rust
// commands.rs — 1 generic command
#[tauri::command]
pub fn invoke_tool(
    state: State<'_, Arc<EngineState>>,
    name: String,
    params: Value,
) -> Result<Value, String> {
    let handler = state.tool_registry.get(&name)
        .ok_or_else(|| format!("Unknown tool: {name}"))?;
    handler(state.engine.clone(), params).map_err(|e| e.to_string())
}
```

### ToolRegistry ownership
Currently `ToolRegistry` lives in `wm_core::mcp::transport`. Tauri imports `wm_core`. The registry needs to be accessible from both the MCP server setup and the Tauri setup. Options:
- Store `Arc<ToolRegistry>` inside `EngineState` itself
- Store it as a separate Tauri managed state alongside `EngineState`
- Build it from `EngineState` on demand (cache)

The cleanest approach: add `tool_registry: Arc<ToolRegistry>` as a field on `EngineState`, populated during `init_engine()`.

### Frontend mapping
The Angular frontend currently calls `invoke('search', { payload: { q: 'test' } })`. To avoid changing the frontend, either:
- Keep the old command names as thin wrappers that call `dispatch_tool` internally
- Or rename to `invoke('invoke_tool', { name: 'wm_search.query', params: { q: 'test' } })` and update the Angular service layer

## Open Questions

- [ ] Should old command names be kept as aliases (backward compat) or should the Angular service layer be updated to use the new generic command?
- [ ] Where exactly should `ToolRegistry` live — inside `EngineState` or as separate Tauri managed state?
- [ ] Should the `dispatch_tool` function live in `wm_core::mcp::transport` or be re-exported from `wm_core` top-level?

## Related Specs

- [Enterprise-Grade Architecture](../conventions/enterprise-grade.md) — D1 locks Tauri as primary, this spec refines the dispatch layer
