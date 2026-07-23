---
title: T1: Replace proxy with direct handlers
type: task
status: done
priority: high
tags: [from-spec, spec/mcp-direct-handlers]
---

## Description

Replace the proxy handlers in `wm-cli mcp` with direct in-process handler registration via `register_all_tools()`.

## Acceptance Criteria

- [ ] AC-1: `wm-cli mcp` starts without wm-server running and tools respond correctly
- [ ] AC-4: `mcp_proxy.rs` and its `STATIC_TOOLS` constant deleted
- [ ] AC-10: `tools/list` from `wm-cli mcp` matches `tools/list` from wm-server (names, descriptions, inputSchemas)

## Fulfills

- FR-1: `wm-cli mcp` creates EngineState from project root and registers tool handlers in-process
- FR-2: `register_all_tools()` called on registry with in-process engine (same as wm-server)
- FR-3: `mcp_proxy.rs` deleted; `STATIC_TOOLS` and proxy handler code removed

## Files

- `apps/wm-cli/src/main.rs` — `Commands::Mcp` handler
- `apps/wm-cli/src/mcp_proxy.rs` — delete entirely
- `apps/wm-core/src/engine/main_engine_factory.rs` — `MainEngine::with_root()` or equivalent for EngineState creation
