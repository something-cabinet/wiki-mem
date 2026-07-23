---
title: Graph UI Fix — Combined Spec
type: spec
---

---
title: Graph UI Fix — Combined Spec
type: spec
tags: [spec, graph, web-ui, interaction, colors, theming]
---

## Overview

This spec consolidates all graph view improvements identified during the design review. It covers: interaction model fixes (drag→navigate, pan/drag conflict), color token theming (node colors, edge colors, legend), interaction upgrades (pointer events, ResizeObserver), and polish issues (centering force, node size, spacing slider, labels).

The graph view renders via Canvas 2D or WebGL (currently unreachable). Layout uses fjadra in Rust via Tauri IPC. CSS has full `--page-type-*` and `--edge-type-*` token definitions that the Canvas 2D renderer does not fully consume.

## Locked Decisions

- D1: Combined spec — all graph bugs (interaction + polish)
- D2: Fix Canvas 2D + wire WebGL renderer reachable via toggle
- D3: Refactor to pointer events (not distance tracking)
- D4: Include color token + legend fixes in spec
- D5: WebGL toggle via Settings, default auto-detect >500
- D6: Extract `GraphColorService` for parsed colors + theme reactivity
- D7: Include graph loading/error/empty states (verify existing)
- D8: Include touch handlers — refactor existing to pointer events
- D9: Instantiate `ResizeObserver` for responsive canvas
- D10: Drop web-mode blank fix (Tauri-only, not a bug)
- D11: fjadra-only layout, remove d3-force
- D12: Include P1–P3 (centering force, node size, spacing slider)
- D13: Fix `--edge-type-*` token naming (hyphen→underscore) + wire
- D14: Label quality pass (font constant, LOD, contrast, background)
- D15: Floating canvas toolbar for spacing/zoom controls
- D16: Fully wire WebGL renderer data binding

## Requirements

### Functional Requirements

**FR-1: Pointer Events Interaction Model**
Replace mousedown/mouseup/mousemove/click handlers with unified pointerdown/pointerup/pointermove handlers. This fixes C2 (drag→navigate) and C3 (pan/drag conflict with d3-zoom).

**FR-2: ResizeObserver Instantiation**
Instantiate the declared-but-never-used ResizeObserver. Canvas must resize when sidebar collapses or window changes.

**FR-3: Node Colors via `--page-type-*` Tokens**
Canvas 2D `nodeColor()` must use distinct `--page-type-{type}` tokens for all 8 types. Currently 4 types map to `--accent`.

**FR-4: Color Legend Fix**
`buildPageTypes()` must not double-wrap `oklch()` values. Swatches must render visible.

**FR-5: Theme-Change Re-render**
Colors must recompute when dark/light theme toggles.

**FR-6: Edge Colors via `--edge-type-*` Tokens**
Rename `--edge-type-depends-on` → `--edge-type-depends_on` to match runtime values. Wire into Canvas 2D edge rendering.

**FR-7: GraphColorService**
Service providing parsed oklch colors per type, emitting on theme change. Both renderers subscribe.

**FR-8: Centering Force (P1)**
fjadra layout must include a gentle centering force for degree-0 nodes.

**FR-9: Node Size Increase (P2)**
Increase base `nodeRadius()` for readable labels at default zoom.

**FR-10: Spacing Slider Affects All (P3)**
Slider controls global repulsion/temperature, not just connected pairs.

**FR-11: Graph States**
Verify loading/error/empty states work with fjadra-only pipeline. Fix error state for fjadra layout failure.

**FR-12: Edge Label Quality (D14)**
Centralize font constant, dark-mode contrast, label background, review LOD thresholds.

**FR-13: Graph Header (D15)**
Move spacing slider + zoom controls to floating canvas toolbar. Header = title + stats only.

**FR-14: WebGL Toggle + Data Binding (D16)**
Settings toggle, auto-detect >500. Wire remaining data bindings (sizes, labels, camera). Move WebGL detection to data-arrival time.

**FR-15: Remove d3-force (D11)**
Remove d3-force simulation. Render pipeline: fjadra positions → applyPositions() → trigger render(). Make render() public or add trigger method.

### Non-Functional Requirements
- NFR-1: Color resolution sub-millisecond per node
- NFR-2: GraphColorService works with Angular signals/effects
- NFR-3: fjadra re-render within one animation frame (16ms)
- NFR-4: All existing E2E tests pass

## Acceptance Criteria

- [ ] AC-1: Dragging a node does not navigate to its page
- [ ] AC-2: Double-clicking a pinned node unpins it
- [ ] AC-3: Dragging a node does not also pan the view
- [ ] AC-4: Canvas resizes when sidebar collapses
- [ ] AC-5: All 8 page types render with distinct, theme-appropriate colors in light and dark mode
- [ ] AC-6: Legend swatches show correct colors, not transparent
- [ ] AC-7: Toggling dark mode re-renders graph with correct colors
- [ ] AC-8: Edges render in type-distinct colors, not all gray
- [ ] AC-9: Unconnected nodes remain visible (do not drift to edge)
- [ ] AC-10: Node labels readable at default zoom (base radius increased)
- [ ] AC-11: Spacing slider changes spacing for all nodes
- [ ] AC-12: Error state shows when fjadra layout fails
- [ ] AC-13: Edge type labels have background for readability
- [ ] AC-14: Graph header shows title + stats only
- [ ] AC-15: WebGL toggle in Settings switches renderer on next load
- [ ] AC-16: WebGL renderer shows correct colors, sizes, and labels
- [ ] AC-17: d3-force dependency removed from the project
- [ ] AC-18: Graph renders correctly in Tauri mode end-to-end

## Scenarios

### Scenario 1: User drags a node
**Given** the graph is open with nodes
**When** the user drags a node to reposition it
**Then** the node follows the pointer, pan does not activate, and releasing does not navigate

### Scenario 2: User toggles dark mode
**Given** the graph is open with light-theme colors
**When** the user toggles dark mode
**Then** nodes, edges, and legend update to dark-theme colors immediately

### Scenario 3: Unconnected node stays visible
**Given** the graph has a degree-0 node
**When** fjadra layout settles
**Then** the deg-0 node is visible within the viewport

### Scenario 4: Large graph triggers WebGL
**Given** >500 graph nodes
**When** the graph loads
**Then** WebGL renderer activates automatically with correct colors and labels

### Scenario 5: Canvas resizes with sidebar
**Given** the graph view is open
**When** the sidebar collapses
**Then** the canvas resizes to fill the new width

## References
- `apps/wm-web/src/libs/graph/canvas-graph.directive.ts` — 493 lines, all interaction + rendering
- `apps/wm-web/src/app/views/graph/graph-view.component.ts` — 276 lines, view + fjadra IPC
- `apps/wm-web/src/libs/graph/webgl-graph.renderer.ts` — 581 lines, unreachable regl renderer
- `apps/wm-web/src/styles.css:60-75, 119-132` — CSS token definitions
- `wiki:specs:graph-ui-polish` — Existing spec (superseded by this combined spec)
