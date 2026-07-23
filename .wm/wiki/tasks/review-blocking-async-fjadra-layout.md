---
title: "Fix: fjadra compute_layout blocks Tauri async runtime"
type: task
status: done
spec: specs/webgl-graph-rendering
tags: [review, backend, fjadra, async, performance]
priority: critical
relates_to:
  - {type: implements, target: wiki:specs:webgl-graph-rendering}
  - {type: depends_on, target: wiki:tasks:fjadra-layout}
---

# Fix: fjadra `compute_layout` blocks Tauri async runtime

## Description

The `compute_layout` Tauri command in `apps/wm-web/src-tauri/src/commands.rs` runs 300 iterations of `sim.tick(1)` in a synchronous loop inside an `async fn`. This blocks the entire Tauri async runtime for the duration of the simulation (100–500ms depending on graph size).

## Location

`apps/wm-web/src-tauri/src/commands.rs` — `compute_layout` function

## Acceptance Criteria

- [ ] Add `tokio::task::yield_now().await;` every tick or every N ticks to yield the runtime
- [ ] OR spawn the simulation on `tokio::task::spawn_blocking()` and emit events from there
- [ ] Verify other Tauri commands are not delayed during layout computation
- [ ] Confirm progressive position events (`graph-coarse`, `graph-refine`, `graph-settled`) still fire correctly

## References

@wiki/tasks/fjadra-layout
