---
id: wiki:specs:graph-and-ui-fix
title: Graph Fix + UI Polish — Combined Spec
type: spec
status: approved
tags: [spec, graph, ui, canvas, fjadra, spartan-ui]
---
id: wiki:specs:graph-and-ui-fix


## Overview

This spec consolidates two workstreams identified during comprehensive audits of the graph visualization and UI component usage. The graph view is currently non-functional (WebGL/regl renderer has never produced a correct frame, layout backend returns a golden-angle spiral stub). The UI has 30+ issues discovered during a SimUI/helm design review, including completely unstyled dialogs, dead theme toggle, and page-type model mismatch.

**Graph Fix:** Drop unreachable WebGL/regl renderer, restore Canvas 2D rendering with correct interaction layer, implement real fjadra force-directed layout via HTTP SSE streaming.

**UI Polish:** Fix P1-P2 issues across dialog styling, theme toggle, page-type picker, dead component cleanup, color consistency, accessibility, and code quality.

## Locked Decisions

- D1: **One combined spec** — both workstreams in a single spec
- D2: **Drop regl WebGL, restore Canvas 2D** — regl is unmaintained/WebGL1, never worked, overkill for 391 nodes. Keep renderer interface seam for future PixiJS swap at measured scale.
- D3: **All P1 + P2 UI issues** in scope, P3 deferred
- D4: **Implement fjadra** in server with full two-phase streaming (coarse → refine → settled)
- D5: **Replace render delegate only** — keep current interaction layer (pointer events, hit-testing, zoom/pan), replace WebGL render calls with Canvas 2D drawing

## Requirements

### Functional Requirements — Graph Fix

**FR-1: Canvas 2D Renderer**
Replace WebGL/regl rendering in `canvas-graph.directive.ts` with Canvas 2D drawing. Keep interaction layer (pointer events, hit-testing, zoom/pan transform, resize observer, label overlay). Must render:
- Nodes as filled circles with size proportional to degree, colored by `--page-type-*` CSS tokens
- Edges as lines between source/target with color from `--edge-type-*` CSS tokens and width by priority
- Hit-testing must match rendered positions at any zoom level

**FR-2: fjadra Server-Side Layout**
Implement real force-directed layout in `apps/wm-server/src/routes/layout.rs`:
- Add fjadra crate dependency to `wm-server`
- `POST /api/graph/layout` — accept nodes + edges + width + height, start simulation, return job_id
- `GET /api/graph/layout/{job_id}/events` — SSE stream with two-phase progressive positions:
  - `graph-coarse` — cluster centers within 50-100ms
  - `graph-refine` — per-node batches (~5k nodes/batch)
  - `graph-settled` — final positions when alpha < threshold
- Forces: many-body repulsion, link attraction, center gravity, collision

**FR-3: Graph Data Pipeline Fix**
- Add `degree` field to `/api/graph/full` response
- Serialize `edge_type` as snake_case to match CSS token naming (`depends_on`, not `DependsOn`)
- Fix edge source/target IDs to be consistent (string IDs matched to node index)
- Build node index map for O(1) lookups instead of O(E·N)

**FR-4: Theme-Responsive Colors**
- Graph must re-render when dark/light theme toggles
- On theme change: re-read CSS tokens, re-render canvas
- Both graph and legend update immediately

**FR-5: Spacing Slider**
- Slider in floating toolbar sends link distance parameter to layout POST
- Must provide visual feedback (repaint when value changes, even if post-layout)

### Functional Requirements — UI Polish (P1)

**FR-6: Fix All Dialogs (P1-1)**
- All 6 dialog instances (`pages-view.component.ts:94`, `page-dialogs.component.ts:28`, `:71`, `memory-view.component.ts:83`, `:118`, `:153`) must use element-form `<hlm-dialog-content *brnDialogContent>` instead of attribute-form `<div hlmDialogContent>`
- Add `HlmDialogContent` import to each component's `imports`
- Remove fallback `<div class="fixed" *brnDialogContent>` workaround from `page-dialogs.component.ts`

**FR-7: Fix Sidebar Theme Toggle (P1-2)**
- Remove `(click)="theme.toggle()"` from label wrapping `hlm-switch` in layout footer
- Settings view: wire label click to `theme.toggle()` or remove `cursor-pointer`
- Extract shared `ThemeToggleComponent` used in both locations

**FR-8: Fix Page-Type Picker (P1-3)**
- Generate type select options from shared `PAGE_TYPES` constant (export from `graph-color.service.ts:4-13`)
- Must include all 7 canonical types + memory: `task, spec, concept, pattern, decision, howto, reference, memory`
- Remove hardcoded `['Default', 'Task', 'Concept', 'Project', 'Note']` in both picker locations

### Functional Requirements — UI Polish (P2)

**FR-9: Dead hlmAlertDesc Selector (P2-1)**
- Fix `<p hlmAlertDesc>` → `<p hlmAlertDescription>` in `tasks-view.component.ts:31` and `search-view.component.ts:80`
- Add `HlmAlertDescription` import to those components

**FR-10: Delete Dead wm-* Components (P2-3)**
- Delete unused `wm-button`, `wm-input`, `wm-badge`, `wm-card`, `wm-dialog`, `wm-select`, `wm-accordion` from `libs/ui/`
- Keep `WmSpinner` (it's in use); optionally rename to helm naming convention

**FR-11: Page-Type Color System (P2-4)**
- Extend badge variants with type colors: `typeBadgeClass(type)` using `--page-type-*` tokens
- Apply type-colored badges in page list, detail header, search results, and task board

**FR-12: Status Color Semantics (P2-5)**
- Fix task board status colors: `urgent` → destructive/red, `todo` → muted, `in-progress` → info, `blocked` → destructive outline

**FR-13: Unify Focus Ring System (P2-6)**
- Scope global `:focus-visible` outline to elements helm doesn't manage, or remove and let components own focus

**FR-14: Silent Delete Error (P2-7)**
- Add error slot to delete dialog or fire toast on delete failure

**FR-15: Graph Keyboard Access (P2-8)**
- Minimum viable: add adjacent hidden node list as links, `tabindex="0"` + arrow-key pan / +/- zoom on canvas
- `role="img"` → `role="application"` with instructions

### Non-Functional Requirements

- NFR-1: Canvas 2D rendering stays above 30fps at 500 nodes with edges
- NFR-2: fjadra layout converges within 2 seconds for 400-node graph
- NFR-3: No regressions in existing E2E test suite
- NFR-4: All dialog changes visually verified (not just compiled)

## Acceptance Criteria

- [ ] AC-1: Graph renders nodes at correct positions with degree-scaled radii
- [ ] AC-2: Graph renders edges between connected nodes with type-distinct colors
- [ ] AC-3: Pan, zoom, and drag produce immediate visual feedback (not static after settle)
- [ ] AC-4: Dragging a node does not navigate to its page
- [ ] AC-5: Dragging does not also pan the view
- [ ] AC-6: Double-tapping a pinned node unpins it
- [ ] AC-7: Canvas resizes when sidebar collapses (ResizeObserver works)
- [ ] AC-8: fjadra layout returns distinct, non-spiral positions
- [ ] AC-9: Unconnected nodes remain visible (do not drift to edges)
- [ ] AC-10: Theme toggle re-renders graph with correct node + edge colors
- [ ] AC-11: All 8 page types render with distinct, theme-appropriate colors
- [ ] AC-12: Spacing slider value is sent with layout request
- [ ] AC-13: All 6 dialogs render with bg-popover, rounded-xl, ring, shadow, padding
- [ ] AC-14: Theme toggle switch in sidebar works on first click every time
- [ ] AC-15: Page-type picker shows all 8 canonical types
- [ ] AC-16: `hlmAlertDescription` renders with correct alert styling (not plain text)
- [ ] AC-17: `wm-*` dead components are removed from the file system
- [ ] AC-18: Type-colored badges appear in page list and search results
- [ ] AC-19: Urgent tasks render with destructive color on the board
- [ ] AC-20: Delete failure shows feedback to the user
- [ ] AC-21: `regl`, `pixi.js`, `@types/pixi.js` dependencies removed from package.json
- [ ] AC-22: O(E·N) edge ID lookups fixed (build reverse index map)
- [ ] AC-23: `degree` field present in graph full endpoint response

## Scenarios

### Scenario 1: User opens graph view
**Given** the wiki has 300+ pages with edges
**When** the user navigates to the graph view
**Then** loading indicator appears, then nodes and edges render at correct positions with degree-scaled sizes and type-distinct colors, toolbar is visible

### Scenario 2: User drags a node
**Given** the graph is rendered
**When** the user pointer-downs on a node, drags it, then releases
**Then** the node follows the pointer during drag, stays at the final position, canvas does not pan, and releasing does not navigate

### Scenario 3: User pans and zooms
**Given** the graph is rendered
**When** the user wheel-zooms or pointer-drags on empty space
**Then** the view transforms with immediate visual feedback, labels reposition correctly

### Scenario 4: User toggles dark mode
**Given** the graph is rendered
**When** the user toggles dark mode via sidebar or settings
**Then** nodes, edges, and legend update to dark-theme colors immediately without page reload

### Scenario 5: User opens a dialog
**Given** any view with a create/edit/delete dialog
**When** the user triggers the dialog
**Then** the dialog opens with correct background, padding, border-radius, shadow, and close button, over a semi-transparent scrim

### Scenario 6: User creates a page
**Given** the create page dialog is open
**When** the user opens the type dropdown
**Then** all 8 canonical types are available (task, spec, concept, pattern, decision, howto, reference, memory)

## Technical Notes

### Canvas 2D Renderer Implementation
- Keep the current `canvas-graph.directive.ts` structure: `screenToGraph`, `hitTest`, `nodeRadius`, `fitToView`, `render()`, pointer event handlers
- Replace `webgl-renderer!` calls in `render()` with Canvas 2D `ctx` operations:
  - Clear canvas, apply transform, draw edges as lines, draw nodes as filled circles
  - Use `readCssColor()` to get theme-aware colors from `--page-type-*` and `--edge-type-*` tokens
- Remove the `WebglGraphRenderer` import and class instantiation
- Remove `webgl-graph.renderer.ts` file (lives in git history)

### fjadra Implementation
- Add `fjadra = "0.5"` (or latest) to `wm-server/Cargo.toml`
- Use `tokio::task::spawn_blocking` for the force simulation (CPU-bound)
- SSE events via `axum::response::sse`
- Two-phase: first emit all cluster centers (spectral or high-degree backbone), then emit refined batches

### Data Pipeline Changes
- In `graph.rs`: add `degree` field using `petgraph`'s node degree method
- In `graph.rs`: serialize `EdgeType` with `#[serde(rename_all = "snake_case")]`
- In `graph.rs`: build `HashMap<String, NodeIndex>` for O(1) source/target lookups

### UI Fix Ordering
- Fix P1s first (dialogs, theme toggle, page-type) — these directly affect usability
- Then P2s (selector fixes, dead code, color system, a11y)
- P3s are deferred

## Open Questions

- [ ] What exact `fjadra` version is available? Check crates.io. Alternatives: `dyno` or custom force layout?
- [ ] Should spacing slider trigger a re-layout POST or just adjust an existing client-side parameter?
- [ ] How to handle Canvas 2D retina DPR (devicePixelRatio backing store vs CSS px coords)?
