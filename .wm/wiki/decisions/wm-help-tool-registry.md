---
title: wm_help must read tool schemas from ToolRegistry
type: decision
tags:
- decision
- good-call
- mcp
- tools
- schemas
status: approved
relates_to:
  - {type: references, target: wiki:tasks:embed-shim-templates}
---

## Context

The `wm_help` MCP tool used a hardcoded list of tool names and descriptions that was maintained separately from the actual `ToolRegistry`. This list:
- Was always out of sync with actual registered tools
- Did not include parameter schemas (the most useful part for agents)
- Required manual updates whenever a tool was added, renamed, or removed
- Had 50+ entries that duplicated data already stored in the registry

Meanwhile, `ToolRegistry` at `transport.rs:43-44` already stored `descriptions: HashMap<String, String>` and `schemas: HashMap<String, Value>` for every registered tool, populated automatically via `schemars` derive macros. The `list_tools()` method returned the full data including schemas, but `wm_help` never used it.

## Decision

Replace the hardcoded tool list in `wm_help` with a dynamic read from the `ToolRegistry`. Store a snapshot of the registered tool list in `EngineState.tool_list` after all tools are registered, and have `wm_help` read from it.

## Rationale

- Eliminates a maintenance burden (50-line hardcoded list removed)
- `wm_help` now returns parameter schemas automatically — agents can discover required fields
- Schema is always in sync because it's generated from the same `#[derive(JsonSchema)]` annotations
- Matches the "single source of truth" principle already established for other project data

## Consequences

- `EngineState` gained a `tool_list: RwLock<Vec<Tool>>` field, which is populated once at startup
- `register_all_tools()` now snapshots the registry after all domain modules register
- `wm_help` returns `{ name, description, schema }` per tool instead of just `{ name, description }`
- Future tool additions automatically appear in `wm_help` with no additional code changes

## Related

- @wiki/tasks/embed-shim-templates
- `apps/wm-core/src/mcp/tools/project.rs` — wm_help handler
- `apps/wm-core/src/engine/engine_state_mediator.rs` — tool_list field