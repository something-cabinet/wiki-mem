---
title: Fix graph WebGL not rendering — canvas blank despite data loading
type: task
status: todo
priority: urgent
tags: [bug, graph, webgl, urgent]
---

Graph view loads 384 nodes and 25 edges but the WebGL canvas shows nothing. Regl draw commands produce transparent pixels. Canvas has `bg-muted/30` CSS background but WebGL content is invisible.

Investigation so far:
- Graph data loads correctly via /api/graph/full (384 nodes, 25 edges)
- startLayout() fetches positions via POST /api/graph/layout → SSE /api/graph/layout/{job_id}/events → positions arrive
- applyPositions() mutates graphNodes with x/y
- triggerRender() → render() → updateNodes() → drawNodes() / drawEdges() → regl clear + draw commands
- Pixel readback via WebGL readPixels shows [0,0,0,0] — nothing drawn
- WebGL 1.0 context available, regl initializes without errors
- Canvas dimensions correct (678x494 CSS, 1356x988 actual at 2x DPR)

Possible root causes:
1. Regl draw commands may silently fail — buffer upload or shader compilation issue
2. Node coordinates may be outside viewport — fitToView() might not work correctly
3. WebGL context lost between frames — preserveDrawingBuffer may be needed
4. Regl version mismatch with shader syntax
5. fitToView() sets transform but may compute wrong values (all nodes at 0,0 initially)

Files to investigate:
- apps/wm-web/src/libs/graph/webgl-graph.renderer.ts — regl draw commands, buffer management
- apps/wm-web/src/libs/graph/canvas-graph.directive.ts — render pipeline, fitToView
- apps/wm-web/src/app/views/graph/graph-view.component.ts — data flow, positions