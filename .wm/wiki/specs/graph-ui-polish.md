---
id: wiki:specs:graph-ui-polish
title: Graph UI Polish Spec
type: spec
status: draft
tags: [spec, web-ui, graph, ux]
---
id: wiki:specs:graph-ui-polish

## Overview

The graph view uses a force-directed layout (fjadra in Rust via Tauri IPC). Three UX issues make the graph hard to read: unconnected nodes drift to the viewport edges, node sizes are too small for labels to be legible, and the spacing slider only controls repulsion between connected nodes instead of all nodes.

## Issues

### P1: Unconnected Nodes Drift Too Far

**Problem:** Nodes that have no edges to other nodes receive no attractive force from the layout. Over iterations they drift toward the edge of the viewport or beyond, becoming invisible and unusable.

**Root cause:** The force-directed layout only applies repulsion and attraction between connected nodes. Isolated nodes (with degree 0) only receive the base repulsion force with no centering force to keep them in view.

**Fix:** Apply a gentle centering force (gravity) that pulls all nodes toward the center of the viewport. This keeps isolated nodes visible while connected nodes still form their natural clusters. The gravity should be weak enough that it doesn't collapse the connected clusters.

### P2: Node Size Too Small

**Problem:** Default node rendering size makes labels unreadable at normal zoom levels.

**Fix:** Increase the base node radius. The current size likely comes from the SDF text atlas scale or the rendering point size.

### P3: Spacing Slider Should Control All Nodes

**Problem:** The spacing slider currently only adjusts the repulsion constant (`k`) for connected node pairs. Unconnected nodes (which have no pair relationship) are unaffected.

**Fix:** The spacing slider should adjust a global repulsion or temperature parameter that affects ALL nodes equally, not just connected pairs.

## Technical Notes

### Layout Pipeline
The graph layout runs in Rust via fjadra:
1. Frontend sends nodes + edges via IPC `computeLayout`
2. Rust runs fjadra force simulation
3. Positions streamed back via `graph-coarse`, `graph-refine`, `graph-settled` events
4. Canvas renders via regl (WebGL)

### Force Parameters (fjadra)
- **Link force** — attraction along edges (connected nodes)
- **Many-body force** — repulsion between ALL nodes (currently not used?)
- **Center force** — gravity toward center (currently missing)
- **Repulsion constant `k`** — adjusted by slider (currently only affects link force, not many-body)

Likely the spacing slider controls `link distance` or `strength` on the link force, not the many-body force. Fix by wiring it to control the many-body force strength instead, or adding a center force.

### Node Rendering
Rendered in `webgl-graph.renderer.ts` via regl. Node size comes from the point/vertex shader or the SDF text atlas. Check `nodeRadius` or similar constants.

## References
- `apps/wm-web/src/libs/graph/webgl-graph.renderer.ts` — WebGL rendering
- Tauri IPC: `computeLayout` → fjadra layout events
