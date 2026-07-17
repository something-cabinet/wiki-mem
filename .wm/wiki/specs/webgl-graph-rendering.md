---
title: WebGL Graph Rendering — regl + fjadra
type: spec
tags: [spec, graph, webgl, rendering, performance]
status: draft
---

## Overview

Replace the current Canvas 2D graph renderer with WebGL using **regl** (declarative WebGL) for rendering and **fjadra** (Rust force-directed layout) for position computation at 100k+ node scale.

## Motivation

The current Canvas 2D + d3-force approach works at <500 nodes. Enterprise target is 100k+ nodes. Canvas 2D drops below 60fps at ~5k visible nodes. d3-force ticks take ~150ms at 100k nodes. Both need replacement.

## Architecture

```
Angular (WebView)                         Tauri (Rust)
─────────────────                         ────────────
                                           fjadra force layout
                                           ├── forceManyBody (repulsion)
regl WebGL renderer                        ├── forceLink (spring edges)
  ├── Instanced circles (100k nodes)       ├── forceCenter
  ├── Batched lines (500k edges)           ├── forceCollide
  ├── SDF text (edge labels + LOD)         └── forceCenter
  ├── Position buffer (Float32Array)       Streams positions via Tauri events
  └── Picking texture (click/hover)        │
       ▲                                   ▼
       └──────── IPC event (graph-positions) ──┘
```

### Data Flow

```
1. User opens Graph view
2. Angular calls invoke('compute_layout', { nodes, edges })
3. Rust (fjadra) begins simulation:
   a. Phase 1 (50-100ms): Compute coarse cluster centers → emit event('graph-coarse')
   b. Phase 2 (500ms-2s): Refine per-node positions → emit event('graph-refine', batch)
4. regl renders each batch as Float32Array arrives
5. When simulation settles (alpha < threshold): emit event('graph-settled')
```

## Technology Choices

### regl over PixiJS
- regl is ~30KB gzipped vs PixiJS ~200KB+
- regl is declarative WebGL — write the GLSL, regl handles buffer/state management
- No game-engine abstractions (sprites, textures, filters)
- Built for data visualization (used by observable, vega-lite internals)
- Better TypeScript support

### fjadra over custom
- fjadra is the Rust equivalent of d3-force: forceLink, forceManyBody, forceCenter, forceCollide
- Multi-threaded via rayon (~5ms per tick at 100k nodes)
- Same API patterns as d3-force — familiar force configuration
- Falls back gracefully: CPU-only, no GPU required for layout

### Binary over JSON
- Positions sent as `Float32Array` (800KB for 100k nodes × 2 coords)
- Tauri IPC supports raw binary transfer natively
- No serialization overhead (vs JSON.stringify on 100k objects)

## Requirements

### FR-1: WebGL Node Rendering
- 100k+ nodes rendered as circles using instanced WebGL draw calls
- Node color encodes page_type (16 type palette)
- Node radius encodes degree centrality (clamped 3-15px)
- White stroke outline for readability on dense graphs
- Single draw call for all nodes

### FR-2: WebGL Edge Rendering
- 500k+ edges rendered as lines using indexed geometry
- Edge color encodes edge_type (16 type palette, semi-transparent at 40%)
- Edge width varies by priority (1-3px)
- Single draw call for all edges

### FR-3: Text Labels with LOD
- Edge type labels at midpoint, rotated along line angle
- SDF (Signed Distance Field) text for crisp rendering at any zoom
- LOD levels:
  - `k < 0.5`: No labels
  - `k >= 0.5`: Priority edges only (extends, implements, depends_on, supersedes)
  - `k >= 1.0`: All edges
- White background rect behind each label for readability

### FR-4: Interaction
- Hover: regl picking texture (render node IDs to offscreen texture, read pixel on hover)
- Click: emit node ID (same picking mechanism)
- Drag: update node fx/fy and restart Rust simulation via IPC
- Pan/Zoom: camera matrix in vertex shader (d3-zoom compatible transform)

### FR-5: Layout (Rust/fjadra)
- `POST /api/graph/full` returns all nodes + edges (existing endpoint)
- `invoke('start_layout', { nodes, edges })` begins the force simulation in Rust
- Simulation runs on a background thread (non-blocking)
- Positions streamed via Tauri events in Float32Array batches
- Cancelable: if user navigates away, drop the event listener, Rust detects dropped channel

### FR-6: Hybrid Two-Phase Streaming
- Phase 1 (coarse): Cluster centers within 50-100ms → user sees structure instantly
- Phase 2 (refine): Per-node positions in batches of ~5k nodes → progressive polish
- User can interact (pan/zoom) during streaming — only coarse positions matter for layout

## Performance Targets

| Metric | Current (Canvas 2D) | Target (regl + fjadra) |
|--------|---------------------|----------------------|
| Node render | 60fps @ 500 nodes | 60fps @ 100k nodes |
| Edge render | 60fps @ 2k edges | 60fps @ 500k edges |
| Layout tick | ~150ms @ 100k (d3-force) | ~5ms @ 100k (fjadra) |
| Initial display | <100ms | <100ms (coarse phase) |
| Full convergence | ~5s @ 500 nodes | ~2s @ 100k nodes |

## Implementation Phases

### Phase 1: regl Node + Edge Rendering (replace Canvas 2D)
1. Install regl + @types/regl
2. Create `WebglGraphRenderer` class with:
   - regl instance on the canvas element
   - Node circle shader (instanced, position + color + radius attributes)
   - Edge line shader (indexed, position + color attributes)
3. Wire into `CanvasGraphDirective` — replace `render()` body
4. Keep Canvas 2D as fallback for <500 nodes (detect and switch)
5. Build check

### Phase 2: SDF Text Labels
1. Create SDF font texture atlas (load pre-rendered glyphs)
2. Add text label shader (batched quads with texture sampling)
3. LOD logic in the render loop
4. Build check

### Phase 3: Interaction (Picking)
1. Offscreen picking texture (render node IDs as colors)
2. Hover → read pixel → emit nodeId
3. Click → read pixel → emit nodeClick
4. Camera matrix (d3-zoom compatible) in vertex shader
5. Build check

### Phase 4: Rust/fjadra Layout
1. Add fjadra dependency to wm-tauri
2. Create `start_layout` Tauri command
3. Background thread with force simulation
4. Batched position streaming via events
5. Cancelation on navigate-away
6. Build check

### Phase 5: Two-Phase Streaming
1. Coarse phase: spectral clustering or high-degree backbone
2. Refine phase: per-batch position streaming
3. Frontend buffer management (grow Float32Array)
4. Smooth animation between coarse and refine positions
5. Build check

## Technical Notes

### regl Shader Outline

```typescript
// Node shader
const drawNodes = regl({
  vert: `
    attribute vec2 position;
    uniform float u_scale;
    varying vec3 v_color;
    void main() {
      gl_PointSize = u_scale * nodeRadius;
      gl_Position = vec4(position, 0, 1);
      v_color = nodeColor;
    }
  `,
  frag: `
    precision mediump float;
    varying vec3 v_color;
    void main() {
      // Circle via fragment discard
      vec2 coord = gl_PointCoord - vec2(0.5);
      if (dot(coord, coord) > 0.25) discard;
      // Stroke
      float dist = length(coord);
      if (dist > 0.4) gl_FragColor = vec4(1,1,1,1);
      else gl_FragColor = vec4(v_color, 1);
    }
  `,
  attributes: {
    position: nodes.map(n => [n.x, n.y]),
    nodeRadius: nodes.map(n => Math.max(3, Math.min(15, n.degree * 0.5 + 3))),
    nodeColor: nodes.map(n => hexToRgb(nodeColorMap[n.page_type] || '#666')),
  },
  count: nodes.length,
});
```

### fjadra Force Configuration

```rust
use fjadra::{Force, Link, ManyBody, Center, Collide, Simulation};

fn force_layout(nodes: &mut [NodeData], edges: &[(usize, usize)], width: f32, height: f32) {
    let mut sim = Simulation::new(nodes.iter().map(|n| n.id))
        .force(Force::many_body().strength(-200.0))
        .force(Force::link(edges.iter().copied()).distance(80.0).strength(0.3))
        .force(Force::center(width / 2.0, height / 2.0))
        .force(Force::collide(10.0))
        .alpha_decay(0.02)
        .velocity_decay(0.3);

    // Tick until settled or max iterations
    while sim.alpha() > 0.001 {
        sim.tick();
        // Every 10 ticks, emit current positions
        if sim.tick_count() % 10 == 0 {
            let positions: Vec<f32> = sim.node_positions()
                .flat_map(|p| vec![p.x, p.y])
                .collect();
            // emit via Tauri event
        }
    }
}
```

### Event-Based Position Streaming

```typescript
// Frontend: listen for position batches
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<{ batchIndex: number; positions: number[]; isComplete: boolean }>(
  'graph-positions',
  (event) => {
    const { batchIndex, positions, isComplete } = event.payload;
    updateNodePositions(batchIndex, new Float32Array(positions));
    render();
    if (isComplete) unlisten();
  }
);
```

### Backward Compatibility
- Canvas 2D renderer kept as fallback for <500 nodes
- regl renderer only activates when node count > threshold (configurable)
- Both share the same input data format (GraphNode[], GraphEdge[])
- Same interaction API (nodeClick, nodeHover)

## Out of Scope
- WebGL 3D rendering (three.js) — 2D knowledge graphs don't benefit from z-axis
- Full PixiJS migration — regl is lighter and more appropriate
- GPU compute for layout (wgpu) — fjadra CPU is fast enough; GPU path is future work
- Animation between layout states — Phase 5 covers coarse→refine transitions

## Open Questions
- [ ] Should we use regl's `buffer` API or raw WebGL for position updates? (regl buffers are easier but slower to update)
- [ ] At what node count should we switch from Canvas 2D to regl? (500, 1000, 5000?)
- [ ] Should SDF text be pre-rendered atlas or runtime-generated?
- [ ] fjadra vs custom Barnes-Hut implementation for the force layout?
