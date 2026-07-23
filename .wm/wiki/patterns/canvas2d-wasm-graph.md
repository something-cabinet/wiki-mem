---
title: Pattern: Canvas 2D + WASM Force-Directed Graph
type: pattern
status: draft
tags: [pattern, graph, canvas, wasm, layout]
relates_to:
  - {type: references, target: wiki:specs/graph-and-ui-fix}
  - {type: references, target: wiki:specs/fjadra-wasm-layout}
  - {type: references, target: wiki:specs/graph-edge-arrows}
---


## Problem

Graph visualizations with 300-10k nodes need fast rendering and interactive layout. WebGL/regl adds complexity, debugging difficulty, and maintainability issues for modest node counts. Server-side layout adds HTTP round-trip latency.

## Solution

**Dual approach:** Canvas 2D for rendering, WASM-compiled fjadra for browser-side layout.

### Rendering: Canvas 2D
- Canvas 2D API for drawing nodes (circles) and edges (lines/bezier curves)
- HTML overlay for edge labels (avoids Canvas text rendering complexity)
- Device pixel ratio handled via `ctx.setTransform(dpr, ...)`
- Single `render()` method called on every interaction (pan/zoom/drag)

### Layout: fjadra compiled to WASM
- fjadra (Rust force-directed layout library) compiled via `wasm-pack build --target web`
- Thin wasm-bindgen wrapper crate exposing `SimulationHandle`:
  - `create(nodes, width, height, edges, linkDistance)` — builds simulation
  - `tick(iterations)` — advances the physics simulation
  - `get_positions()` — returns `Float64Array` of `[x0, y0, x1, y1, ...]`
  - `is_finished()` — returns true when alpha has decayed below threshold
- Progressive tick loop: `tick(15)` × 3 per `requestAnimationFrame` frame, yields smoothly
- Dynamically imported as ES module chunk (lazy-loaded, not in main bundle)

### Interaction model
- Two separate `pointermove` handlers: one unconditional for hover (tooltip + cursor), one guarded by `pointerState` for drag/pan
- All positions use `fx/fy` (pinned) when set, falling back to `x/y` (layout positions)
- Spacing slider re-runs the WASM simulation with new `linkDistance` without resetting zoom

### Edge rendering features
- Straight lines for single-direction edges
- Quadratic bezier curves for bidirectional (antiparallel) pairs, offset ±15px perpendicular
- Triangle arrowheads at target endpoint, inset from node edge
- Labels as HTML overlay spans, rotated along edge angle, normalized upright

## When to Use
- Node counts under 10k (Canvas 2D is sufficient)
- Browser-based layout preferred over server round-trip
- Angular frontend with Rust backend

## When Not to Use
- Node counts above 20k+ where Canvas 2D drops below 30fps (consider PixiJS)
- Server-side layout required for collaborative/real-time scenarios
- No WASM support in target browsers

## Related
- @wiki/specs/graph-and-ui-fix
- @wiki/specs/fjadra-wasm-layout
- @wiki/specs/graph-edge-arrows
- @wiki/patterns/critical-patterns
