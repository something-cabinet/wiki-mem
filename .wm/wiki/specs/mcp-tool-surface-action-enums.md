---
title: MCP Tool Surface Refactoring — Action Enums
type: spec
status: approved
tags: [mcp, tools, refactor, api-surface]
---

## Overview

Flatten the MCP tool surface from 78 individual dot-notation tools to ~33 tools by merging CRUD-like domains into single tools with `action` enum variants. This matches Knowns' pattern (`known_doc { action: "list" }`) and improves AI agent discovery by reducing noise in `tools/list`.

## Locked Decisions

- **D1:** Merge CRUD-like domains (page, doc, memory, task, source, template, decision, model, time, index) into action-enum tools.
- **D2:** Keep semantically distinct operations separate (search.query, search.retrieve, graph.stats, graph.path, code.search, code.symbols, etc.)
- **D3:** HTTP API routes (`/api/pages/list`, `/api/search`, etc.) remain unchanged — they call `wm_core` functions directly, not through the MCP registry.
- **D4:** Drop the fake `register_read`/`register_write`/`register_admin` distinction. Replace with a single `register()`. Delete the 225-line `typed.rs` module. Access levels are dead code (identical implementations, `check_permission` never set).
- **D5:** Action names match current tool suffixes (snake_case): `check_ac`, `resolve_all`, `create`, `delete`.
- **D6:** Serde handles parse errors for unknown actions. A fallback match arm adds "Available actions: ..." with the full list.

## Requirements

### Functional Requirements

- **FR-1:** Each merged domain exposes one MCP tool with an `action` string parameter. The action value determines which operation to execute.
- **FR-2:** Each action variant has its own typed parameter set, documented in the JSON schema via `#[serde(tag = "action")]` discriminated union.
- **FR-3:** The existing `register_read`/`register_write`/`register_admin` access level distinction is preserved per action variant, not lost in the merge.
- **FR-4:** The `list_tools` response returns ~33 tools (down from 78). Each tool description lists the available actions.
- **FR-5:** Audit logging in `ToolRegistry.dispatch()` still records the action name (derived from the `action` field, not the trailing dot-segment).

### Non-Functional Requirements

- **NFR-1:** Zero changes to `wm-server` HTTP API routes or handler implementations.
- **NFR-2:** Zero changes to the web UI API service (`api.service.ts`) — it uses HTTP routes, not MCP tools.
- **NFR-3:** Existing in-process MCP tests continue to pass without modification (they test tool behavior, not tool count).

### Out of Scope

- The HTTP API surface (`/api/pages/list` etc.) — no changes.
- The web UI — no changes.
- The `wm-cli` proxy — it auto-discovers tools from the registry, so it adapts automatically.

## Acceptance Criteria

- [ ] **AC-1:** `tools/list` returns ~33 tools (previously 78).
- [ ] **AC-2:** Each merged tool accepts `{"action": "list"}`, `{"action": "get", "id": "..."}`, etc. at the top level.
- [ ] **AC-3:** Calling a merged tool with an invalid action returns a clear error: "unknown action: invalid_action".
- [ ] **AC-4:** Calling a merged tool with valid action but wrong params (e.g., `get` without `id`) returns a field-level error.
- [ ] **AC-5:** All existing MCP tests pass (42/42).
- [ ] **AC-6:** All existing CLI tests pass (31/31).
- [ ] **AC-7:** All existing E2E tests pass (3/3).
- [ ] **AC-8:** All individual tools still work end-to-end through the MCP proxy.

## Tool Map

### Merged (49 tools → 10)

| MCP Tool | Actions |
|----------|---------|
| `wm_page` | list, get, create, update, delete, link, unlink |
| `wm_doc` | list, get, create, update, delete |
| `wm_memory` | list, get, add, update, delete, promote |
| `wm_task` | list, get, create, update, delete, check_ac, uncheck_ac, board, subtask |
| `wm_source` | add, process, complete, error, list, verify, discover, remove, status |
| `wm_template` | list, get, create, run |
| `wm_decision` | create, get |
| `wm_model` | list, status, download, remove |
| `wm_time` | start, stop, add, report |
| `wm_index` | rebuild, embed, status |

### Kept Separate (~23 tools)

| Tool | Rationale |
|------|-----------|
| `wm_search.query` | Very different params from retrieve/resolve |
| `wm_search.retrieve` | Context assembly with token budget + BFS |
| `wm_search.resolve` | ID lookup |
| `wm_graph.stats` | No params |
| `wm_graph.neighbors` | Single ID, depth |
| `wm_graph.path` | Start + end |
| `wm_graph.subgraph` | Center + depth |
| `wm_code.search` | Regex text search |
| `wm_code.symbols` | AST symbol lookup |
| `wm_code.deps` | Import analysis |
| `wm_lint.check` | — |
| `wm_lint.fix` | — |
| `wm_validate.check` | — |
| `wm_log.recent` | — |
| `wm_log.since` | — |
| `wm_log.filter` | — |
| `wm_project.status` | — |
| `wm_project.detect` | — |
| `wm_project.set` | — |
| `wm_ref.*` (3) | — |
| `wm_skill.trigger` | — |
| `wm_initial` | — |
| `wm_help` | — |

### Registration Example

```rust
#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action")]
enum WmPageAction {
    #[schemars(description = "List all wiki pages")]
    List {
        #[schemars(description = "Filter by page type")]
        r#type: Option<String>,
        #[schemars(description = "Max results")]
        limit: Option<usize>,
    },
    #[schemars(description = "Get page content by ID")]
    Get { id: String },
    #[schemars(description = "Create a new wiki page")]
    Create {
        path: String,
        title: String,
        content: Option<String>,
    },
    // ...
}

registry.register(
    "wm_page",
    "Page CRUD operations: list, get, create, update, delete, link, unlink",
    move |input: WmPageAction| match input {
        WmPageAction::List { r#type, limit } => handle_list(engine, r#type, limit),
        WmPageAction::Get { id } => handle_get(engine, id),
        WmPageAction::Create { path, title, content } => handle_create(engine, path, title, content),
        // ...
    },
);
```

## Scenarios

### Scenario 1: Agent Calls Merged Tool
**Given** an AI agent wants to get a page
**When** it calls `wm_page` with `{"action": "get", "id": "wiki:concepts:test"}`
**Then** the tool returns the page content, same as calling `wm_page.get` before

### Scenario 2: Agent Calls Merged Tool with Bad Action
**Given** an AI agent calls `wm_task` with `{"action": "fly", "id": "..."}`
**When** the action `fly` doesn't exist
**Then** the tool returns an MCP error: "unknown action: fly"

### Scenario 3: Separate Tools Unchanged
**Given** an AI agent wants to search
**When** it calls `wm_search.query` with `{"q": "graph engine"}`
**Then** the tool works exactly as before — no action enum needed

### Scenario 4: HTTP API Unchanged
**Given** a curl request to `POST /api/pages/list`
**When** the server receives it
**Then** it dispatches directly to the wm-core handler, bypassing the MCP registry entirely

## Technical Notes

- Use `#[serde(tag = "action")]` on the enum for clean discriminated union JSON schema
- The `registered_read`/`register_write` distinction can be preserved by wrapping the action enum with a marker type or by having separate `register_*` calls for read vs write actions within the same domain file
- The proxy in `wm-cli` auto-discovers tools from the registry — no proxy changes needed
- Audit log action extraction: change from `name.split('.').nth(1)` to `params.get("action")`

## Related

- Knowns `known_*` tool pattern (v0.20.5) — reference implementation
- @wiki/learnings/proxy-architecture-single-entrypoint — Current tool architecture
- @wiki/concepts/patterns/critical-patterns — MCP prefix pattern
