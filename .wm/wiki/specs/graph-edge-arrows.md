---
id: wiki:specs:graph-edge-arrows
title: Graph Edge Direction Arrows
type: spec
status: approved
tags: [spec, graph, edges, arrows, visual]
---
id: wiki:specs:graph-edge-arrows


## Overview

Graph edges currently render as straight or curved lines with no direction indicator. Even in directed graphs (where every edge has a source→target orientation), a user cannot tell which direction a relationship points without clicking through to a page. This spec adds triangle arrowheads at the target end of every edge.

## Locked Decisions

- D1: **All edges** get direction arrows
- D2: **Triangle arrowhead** at target end

## Requirements

### Functional Requirements

**FR-1: Arrowhead Rendering**
Every edge draws a filled triangle arrowhead at its target endpoint, pointing in the direction of the edge (source → target). The arrowhead must:
- Be a filled triangle with the tip at the exact target node position
- Have a base width proportional to the edge line width (~3× line width)
- Have a length proportional to the edge line width (~5× line width)
- Color match the edge color (same `--edge-type-*` token)
- Scale correctly with zoom/pan (part of the graph-space drawing)

**FR-2: Bidirectional Edge Arrows**
For antiparallel edge pairs (A→B, B→A), each curved bezier edge gets its own arrowhead at its respective target endpoint, following the curve direction. Arrowhead orientation must follow the tangent of the curve at the endpoint, not the straight-line chord.

**FR-3: Zoom Scaling**
Arrowhead size must scale with the camera zoom (multiply by 1/k) so they appear consistent at any zoom level, just like line widths and node radii.

### Non-Functional Requirements

- NFR-1: Arrowhead rendering must not add measurable frame time (simple triangle fill is GPU-composited by Canvas 2D)
- NFR-2: Arrowheads must not overlap with node body (tip at exact target position, base stays clear of node radius)

## Acceptance Criteria

- [ ] AC-1: Every edge shows a triangle arrowhead at the target end
- [ ] AC-2: Arrowhead color matches the edge type color
- [ ] AC-3: Arrowhead size scales with zoom (proportional to line width)
- [ ] AC-4: Bidirectional curves each have their own arrowhead following the curve tangent
- [ ] AC-5: Arrowheads don't overlap with node body rendering

## Scenarios

### Scenario 1: User views directed edges
**Given** the graph shows 25 edges with various types
**When** the user views the graph at default zoom
**Then** each edge has a triangle arrowhead at its target end, matching the edge color

### Scenario 2: User zooms in
**Given** the graph is rendered with arrowheads
**When** the user zooms in
**Then** arrowheads scale up proportionally (same visual size relative to edges)

### Scenario 3: User views bidirectional pair
**Given** two nodes have edges in both directions (A→B, B→A)
**When** the user views them
**Then** each curved edge has its own arrowhead following the curve at its endpoint

## Technical Notes

### Arrowhead geometry
```
         /\
        /  \
       /    \
      / base \
     /________\
        |
      tip (at target node)
```

In code, compute two points perpendicular to the edge direction at a distance `baseWidth` from the tip, then fill the triangle with the edge color.

For curved edges (bidirectional pairs), compute the tangent of the quadratic bezier at t=1 (the endpoint) to orient the arrowhead correctly.

### Implementation location
`canvas-graph.directive.ts` — `render()` method, in the edge drawing block after `ctx.stroke()`.

## Open Questions

- [ ] Should the arrowhead base be inset slightly from the exact target position so it doesn't overlap with the node circle?
