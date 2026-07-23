---
title: Graph Bugs & Review Fixes
type: spec
tags: [spec, graph, bugs, review]
status: approved
relates_to:
  - {type: references, target: wiki:tasks:task-graph-cycle-detected}
  - {type: answers, target: wiki:tasks:task-graph-cycle-detected}
---

---
title: "Graph Bugs & Review Fixes"
type: spec
status: approved
tags: [spec, graph, bugs, review]
---

## Overview

Resolve a batch of identified graph issues: cycle detection, unregistered edge type, empty Tauri graph, event listener race condition, unused variable, and async blocking in layout computation.

## Locked Decisions

- D1: **Findings grouped by domain** — all graph bugs and review findings are handled in one batch spec to reduce per-item overhead. Each issue maps to an AC.
- D2: **Fix first, then formalize** — low-ambiguity fixes proceed directly; only items with design choices need separate spec pages.

## Requirements

### FR-1: Resolve graph cycle detection warning
The graph engine detects cycles but only logs a warning. The cycle is likely a bidirectional `relates_to` link. Confirm the cycle source is benign and suppress the warning noise, or document if a real cycle exists.

**Resolved:** @wiki/tasks/task-graph-cycle-detected — two cycles found, both intentional mutual references. The `info!()` log message already describes this as expected behavior.

### FR-2: Register `implemented-by` edge type
Custom edge type `implemented-by` is missing from `.wm/config.json` `custom_edge_types`. Edges of this type are silently skipped during graph rebuild. Add the registration.

### FR-3: Fix empty Tauri graph (detect_project_root)
Tauri's `detect_project_root()` fails when the binary is launched outside the project directory, resulting in 0 graph nodes. Fix root detection or add a `--root` override.

### FR-4: Fix graph layout event listener race
`startLayout()` in `graph-view.component.ts` fires `computeLayout` before async Tauri event listeners are registered. Progressive `graph-coarse`/`graph-refine` events may be missed. Fix registration order.

### FR-5: Remove unused `_index` variable in graph.rs
`let _index = &snapshot.1;` in graph.rs signals "intentionally unused" but is still dead weight. Either document why it's kept, or remove it entirely.

### FR-6: Fix fjadra compute_layout blocking Tauri async runtime
The `compute_layout` Tauri command runs synchronous fjadra simulation ticks in an `async fn`, blocking the Tauri runtime for 100-500ms. Move to a background thread (`tokio::spawn_blocking`).

## Acceptance Criteria

- [x] AC-1: Graph cycle warning on startup either resolved or explicitly documented as benign
- [ ] AC-2: `implemented-by` edges appear in graph neighbors and stats
- [ ] AC-3: Tauri app shows graph nodes on startup (or `--root` flag works)
- [ ] AC-4: Graph layout `graph-coarse`/`graph-refine` events are always received
- [ ] AC-5: No `_index` dead variable in graph.rs (removed or commented)
- [ ] AC-6: fjadra `compute_layout` does not block the Tauri async runtime

## Scenarios

### Scenario 1: Startup with Tauri binary outside repo
**Given** the Tauri binary is launched from `target/debug/`
**When** the app initializes
**Then** graph shows 244+ nodes (or user can pass `--root /path/to/repo`)

### Scenario 2: Graph rebuild with implement-by edges
**Given** `.wm/config.json` has `implemented-by` in `custom_edge_types`
**When** graph rebuilds
**Then** edges of type `implemented-by` are included in graph

## Technical Notes

### Files likely to change

| File | Issue |
|---|---|
| `apps/wm-web/src-tauri/src/lib.rs` | detect_project_root fix |
| `apps/wm-web/src-tauri/src/commands.rs` | fjadra blocking fix |
| `.wm/config.json` | register `implemented-by` edge type |
| `apps/wm-web/src/app/views/graph/graph-view.component.ts` | event listener race |
| `apps/wm-core/src/mcp/tools/graph.rs` | dead `_index` variable |
| `apps/wm-core/src/graph/engine.rs` | cycle detection warning |

## Open Questions

- [x] Is the graph cycle real or just bidirectional relates_to? Need to verify graph structure.

**Answer:** Two cycles found, both intentional bidirectional references (see @wiki/tasks/task-graph-cycle-detected).