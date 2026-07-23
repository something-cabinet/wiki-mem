---
title: Architectural Refactors — tools.rs Split, Dependency Inversion, Extraction
type: spec
tags: [spec, approved, refactor, architect]
---

## Overview

Apply 6 architect-recommended refactors to improve code organization, reduce duplication, and invert coupling. The centerpiece is splitting the 1969-line `mcp/tools.rs` into per-domain modules.

## Locked Decisions

- D1: tools.rs splits into `mcp/tools/` directory with per-domain modules (search, page, source, graph, lint, validate, index, task, log, time, model, project, misc) — 100-250 lines each
- D2: tools.rs becomes ~80-line delegator re-exporting from domain modules
- D3: Skill → MCP dependency inverted — skill.rs returns `tool_specs()` Vec, MCP registration in tools.rs
- D4: rebuild_memory_index extracted to search.rs (BM25 logic moves, EngineState wrapper stays)
- D5: recover_orphan_timers moves from source.rs to page.rs (operates on tasks, not sources)
- D6: Duplicate BFS path-finding extracted to `graph::find_path()`

## Requirements

### Functional Requirements

- FR-1: tools.rs must delegate to per-domain modules with zero functional change
- FR-2: Skill system must not depend on MCP registration code
- FR-3: rebuild_memory_index BM25 logic must be reusable from search.rs
- FR-4: orphan timer recovery must be co-located with task operations
- FR-5: Duplicate BFS path-finding must have one implementation

### Non-Functional Requirements

- NFR-1: Zero functional changes — all existing tests must pass
- NFR-2: `cargo build` and `cargo test` pass without new warnings
- NFR-3: Each tools/ module must be 100-250 lines max

## Acceptance Criteria

- [ ] AC-1: `mcp/tools/` directory exists with per-domain modules (search.rs, page.rs, source.rs, graph.rs, lint.rs, validate.rs, index.rs, task.rs, log.rs, time.rs, model.rs, project.rs, misc.rs)
- [ ] AC-2: tools.rs is ~80 lines (re-exports from domain modules)
- [ ] AC-3: skill.rs `register_mcp_tools()` replaced with `tool_specs()` data method
- [ ] AC-4: MCP tool registration in tools.rs calls `skill::tool_specs()`, not the reverse
- [ ] AC-5: `search::rebuild_memory_index_from_dir()` exists in search.rs; EngineState wrapper calls it
- [ ] AC-6: `recover_orphan_timers` moved from source.rs to page.rs
- [ ] AC-7: `graph::find_path()` is the single BFS path implementation
- [ ] AC-8: tools.rs:984-1043 and main.rs:1533-1558 both call `graph::find_path()`
- [ ] AC-9: All existing tests pass (cargo test)

## Scenarios

### Scenario 1: Add a new MCP tool
**Given** a developer needs to add a new search-related MCP tool
**When** they open the codebase
**Then** they find it in `mcp/tools/search.rs` (not a 1969-line file), register it via tools.rs delegator

### Scenario 2: Skill system change
**Given** a developer needs to change how skills provide tool metadata
**When** they modify skill.rs
**Then** they don't need to touch MCP registration code — skill.rs only returns data

## Technical Notes

- tools.rs at 1969 lines is the largest file in the codebase
- The `register_mcp_tools()` pattern creates circular-like coupling between skill.rs and tools.rs
- Use `pub mod search; pub use search::*;` pattern in tools.rs for re-exports
- Module names match MCP tool domain prefixes (wm_search → search.rs, wm_page → page.rs, etc.)
- Also add ScorcingConfig unit tests and PageType tests as bonus items