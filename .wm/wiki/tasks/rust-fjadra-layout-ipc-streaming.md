---
title: Rust fjadra Force Layout + IPC Streaming
type: task
status: todo
priority: high
spec: specs/webgl-graph-rendering
tags: [rust, graph, layout, fjadra, tauri]
relates_to:
  - {type: implements, target: wiki:specs:webgl-graph-rendering}
---

## Objective
Replace d3-force with Rust fjadra for force-directed graph layout at 100k+ node scale. Compute positions in wm-tauri, stream to Angular via Tauri events.

## Context
Currently d3-force runs in the browser JS thread. At 100k nodes, a single tick takes ~150ms — unusable. fjadra is a Rust port of d3-force, multi-threaded via rayon, achieving ~5ms/tick at 100k nodes. Layout runs in wm-tauri's background thread, positions streamed via Tauri events.

## Acceptance Criteria
- [ ] Add fjadra dependency to wm-tauri Cargo.toml
- [ ] Create `start_layout` Tauri IPC command accepting nodes + edges
- [ ] Run force simulation on background thread (tokio::spawn)
- [ ] Stream positions via `app_handle.emit("graph-positions", batch)`
- [ ] Hybrid two-phase: coarse cluster centers (50ms) then refined batches
- [ ] Cancelation: drop event listener → Rust detects channel drop
- [ ] Angular WebglGraphRenderer accepts Float32Array position updates
- [ ] Canvas 2D fallback for <500 nodes (unchanged)

## Implementation Notes
```rust
// Force configuration
Simulation::new(nodes)
    .force(Force::many_body().strength(-200.0))
    .force(Force::link(edges).distance(80.0).strength(0.3))
    .force(Force::center(w/2, h/2))
    .force(Force::collide(10.0))
    .alpha_decay(0.02)
    .velocity_decay(0.3)
```
- Phase 1: cluster graph (find high-degree backbone) → emit coarse positions
- Phase 2: per-node batches of 5000 → emit refined positions
- Frontend buffer: pre-allocate Float32Array, append batches
