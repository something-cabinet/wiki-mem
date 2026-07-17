---
title: "Obsidian-like Graph View"
type: spec
tags: [spec, graph, visualization, web-ui, angular, d3, api]
status: draft
---

# Spec: Obsidian-like Graph View

## Overview

Replace the current stat-card + text-input graph view with an interactive force-directed knowledge graph visualization, inspired by Obsidian's graph view. The graph view should render wiki pages as nodes, typed relationships as edges, and support the same interactive gestures users expect: drag, zoom, pan, click-to-navigate, and hover-to-inspect.

### Why Obsidian's model?

Obsidian's graph is the gold standard for personal knowledge graph visualization because it balances aesthetics with utility. It doesn't try to be a full graph analysis tool — it prioritizes spatial browsing, discovery of orphan pages, and understanding the "shape" of a knowledge base at a glance. Our wiki already has the typed graph data; we should surface it visually.

---

## 1. Requirements

### FR-1: Global Graph View

Replace the current stats cards and text-input explorer with a full-screen canvas rendering **all** graph nodes and edges using a force-directed layout.

- **Default view**: All nodes and edges visible when entering the graph tab
- **Physics simulation**: Nodes repel each other; edges act as springs pulling connected nodes together
- **Simulation runs client-side**: Positions computed in the browser, not server-side

### FR-2: Interactive Navigation

- **Pan**: Click-and-drag on empty canvas to pan the viewport
- **Zoom**: Scroll wheel (or pinch) to zoom in/out. Clamp zoom to reasonable bounds (0.1x–4x)
- **Drag nodes**: Click-and-drag a node to reposition it. Node stays pinned at new position (simulation continues around it)
- **Animate**: Smooth transitions when nodes move (simulation tick)

### FR-3: Node Interaction

- **Hover**: Show a tooltip with the page title, page type, and edge count
- **Click**: Open the page detail view (navigate to Pages view with that page pre-selected, or open a slide-out panel)
- **Right-click**: Context menu with actions: "Open in Pages", "Explore Neighbors", "Copy Page ID"

### FR-4: Visual Encoding

- **Node size**: Proportional to the number of connections (degree centrality). More connected = larger node. Clamp to a min/max range.
- **Node color**: Encodes `page_type` (e.g., concept=blue, spec=green, task=amber, memory=purple, rule=red). Use a consistent color palette across the app.
- **Node label**: Show page title text next to nodes. Labels fade in as user zooms in (to prevent clutter at global zoom). Only label the largest/most-connected nodes at default zoom.
- **Edge color**: Encodes `edge_type` (lighter/different hue per type). Edges are semi-transparent to avoid visual overload.
- **Edge thickness**: Uniform or slightly varied by edge weight (if weighted edges exist)

### FR-5: Filtering & Search

- **Page type filter**: Toggle nodes on/off by `page_type`. Checkboxes or pills at the top of the canvas (e.g., "Concepts", "Specs", "Tasks", "Memory", "Rules").
- **Search/focus**: A search bar that highlights matching nodes (by ID or title match) and dims non-matching nodes. Enter searches; selecting a result centers the view on that node.
- **Orphan detection**: Optionally highlight nodes with zero connections (orphan pages).

### FR-6: Local Graph (Neighbor Explorer)

- **Single-node focus**: Double-click a node to switch to "local graph" mode, showing only that node + its immediate neighbors (1–2 hops)
- **Breadcrumb back**: A "Back to full graph" button to return to global view
- **Expand**: Click a neighbor node to re-center the local graph on it
- This replaces the current text-input neighbor explorer entirely

### FR-7: Performance

- **Target**: Smooth 60fps interaction with up to 500 nodes and 2,000 edges
- **Above 500 nodes**: Graceful degradation — remove labels, reduce simulation tick rate, use WebGL renderer if necessary
- **Initial load**: Fetch all graph data once on mount. Use requestAnimationFrame for simulation ticks

### FR-8: Accessibility (Minimum Viable)

- **Keyboard navigation**: Tab through nodes, Enter to select, arrow keys to pan (when no node focused)
- **Color-independent encoding**: Page type also indicated by node shape or a text label, not just color
- **Screen reader**: Canvas is supplemented with a hidden list of nodes for SR users (`role="list"` with `aria-label`)

---

## 2. Technical Approach

### Recommended: D3-force + Canvas (with Angular directive wrapper)

**Why D3-force?**

- Obsidian itself uses d3-force under the hood (it's an Electron app bundling d3)
- d3-force is the most mature and battle-tested force-directed layout library in the JS ecosystem
- It provides exactly the simulation primitives needed: `forceManyBody` (repulsion), `forceLink` (spring edges), `forceCenter` (gravity toward center), `forceCollide` (prevent overlap)
- Excellent TypeScript type definitions (`@types/d3-force`)
- No DOM dependency — simulation is pure data; rendering is separate. This means we can use Canvas (high performance) while d3-force handles the math

**Why Canvas, not SVG?**

- SVG is simpler but struggles above ~200 nodes due to DOM overhead (each node/edge is a DOM element)
- Canvas is a single bitmap; rendering 500+ nodes at 60fps is trivial
- d3-force has no rendering opinion — it outputs `{ x, y }` positions. We render those onto a `<canvas>` element ourselves
- We still use SVG for the toolbar/controls overlay; only the graph itself is canvas

**Why not sigma.js?**

- Sigma.js (v2/v3) uses WebGL which is even faster for very large graphs (10,000+ nodes), but adds ~200KB gzipped, has a steeper learning curve, and is overkill for our expected graph size
- d3-force is ~30KB gzipped and fits our needs perfectly

**Why not ngx-graph?**

- ngx-graph wraps d3 but adds Angular-specific abstractions that may not be compatible with Angular 22
- Less control over rendering; ties us to SVG rendering which doesn't scale well
- More dependencies, less flexibility

### Architecture

```
GraphViewComponent (Angular)
  ├── GraphToolbarComponent (filters, search, mode toggle)
  ├── GraphCanvasDirective (canvas rendering + d3-force simulation)
  │     ├── d3.forceSimulation (physics)
  │     ├── Canvas 2D render loop (requestAnimationFrame)
  │     └── Interaction handlers (zoom/pan/drag/click/hover)
  └── GraphNodeTooltipComponent (hover tooltip)
```

**Key design decisions:**

- **Simulation runs in a Web Worker?** Not needed for <500 nodes. d3-force's tick is ~1ms for this size on modern hardware. If we exceed 1,000 nodes, we can move simulation to a worker.
- **State management**: Graph data lives in `GraphViewComponent` as signals. The canvas directive receives data via `@Input()`. Filters update a `visibleNodes`/`visibleEdges` computed signal.
- **Change detection**: The canvas is imperative (direct DOM manipulation via Canvas 2D API), so it doesn't trigger Angular change detection. We use `ChangeDetectionStrategy.OnPush` on the component and `NgZone.runOutsideAngular()` for the render loop — consistent with the production readiness spec (FR-2).
- **ResizeObserver**: Canvas auto-resizes to fill its container. Use `ResizeObserver` rather than window resize events.

### D3-force configuration (starting parameters)

```typescript
const simulation = d3.forceSimulation(nodes)
  .force('link', d3.forceLink(edges).distance(80).strength(0.3))
  .force('charge', d3.forceManyBody().strength(-200))
  .force('center', d3.forceCenter(width / 2, height / 2))
  .force('collide', d3.forceCollide().radius(d => nodeRadius(d) + 4))
  .alphaDecay(0.02)       // how fast simulation cools
  .velocityDecay(0.3);    // friction
```

These values are tuning starting points; final parameters should be adjustable via Settings.

---

## 3. Data Requirements

### 3.1 New Backend Endpoint: `POST /api/graph/full`

The current `POST /api/graph/neighbors` only returns neighbors of a single node. We need a **full graph dump** endpoint for the global graph view.

**Request:**
```json
{
  "include_edges": true,
  "page_types": null,
  "ids": null
}
```

- `include_edges`: always `true` for visual graph; `false` for stats-only consumers
- `page_types`: optional filter array. If provided, only return nodes of those types (and edges between them). Client-side filtering is also possible, but server-side filtering reduces wire size.
- `ids`: optional array of node IDs. If provided, return only those nodes + edges between them (for local graph mode). Server-side subgraph extraction avoids sending the whole graph for local graph views.

**Response:**
```json
{
  "success": true,
  "nodes": [
    {
      "id": "wiki:concepts:graph-node",
      "title": "Graph Node",
      "page_type": "concept",
      "degree": 12
    }
  ],
  "edges": [
    {
      "source": "wiki:concepts:graph-node",
      "target": "wiki:specs:obsidian-graph-view",
      "edge_type": "depends_on"
    }
  ]
}
```

**Why node IDs as strings in edges, not indices?** Petgraph `NodeIndex` values are not stable across graph rebuilds. String IDs are stable and the client will build its own index map. This also keeps the API graph-agnostic (a Python client shouldn't need to understand petgraph internals).

**Why include `degree` per node?** The degree (connection count) is cheap to compute server-side during serialization and is used client-side for node sizing. Avoids the client having to compute it from edges.

**Performance consideration:** For very large wikis (10,000+ pages), the full graph JSON could be several MB. For now, this is acceptable — our current wikis are in the hundreds. If this becomes an issue later:
- Compress with gzip/brotli (Axum middleware, essentially free)
- Add pagination for the initial load
- Use a streaming JSON response (NDJSON) for progressive rendering

### 3.2 Modified Endpoint: `POST /api/graph/neighbors`

Extend the existing neighbor endpoint to support multi-hop:

```json
{
  "id": "wiki:concepts:graph-node",
  "depth": 2,
  "include_incoming": true
}
```

- `depth`: default `1` (current behavior). Allows "local graph" mode with 2-hop neighborhoods.
- `include_incoming`: return incoming edges (edges pointing TO this node) in addition to outgoing. Current behavior only returns outgoing.

**Response shape unchanged** — still `{ center_id, neighbors: [...] }`, but `neighbors` includes edges at all depths with a `depth` field.

### 3.3 Data volume estimate

| Wiki size | Nodes | Edges | JSON size (approx) |
|---|---|---|---|
| Small (100 pages) | 100 | 200 | ~30 KB |
| Medium (500 pages) | 500 | 1,500 | ~200 KB |
| Large (2,000 pages) | 2,000 | 8,000 | ~1.2 MB |
| Reference: Obsidian vault (5,000 notes) | 5,000 | 20,000 | ~3 MB |

Target: optimize for the small-to-medium case (<500 nodes). At this scale, the full graph round-trip is <100ms on localhost and renders in <1s.

### 3.4 Existing endpoints to keep

- `POST /api/graph/stats` — keep as-is. Still useful for the header/summary bar.
- `POST /api/graph/neighbors` — extend (see 3.2) but keep backward-compatible.

---

## 4. Implementation Phases

### Phase 1: Static Graph Rendering (MVP Visualization)

**Goal:** A canvas shows all nodes/edges with force-directed layout. No interaction yet.

- [ ] Add `POST /api/graph/full` backend endpoint
- [ ] Add `getGraphFull()` to `ApiService`
- [ ] Install `d3-force` and `@types/d3-force` npm packages
- [ ] Create `GraphCanvasDirective` that:
  - Receives `{ nodes, edges }` via `@Input()`
  - Initializes d3-force simulation
  - Renders nodes as circles and edges as lines on a `<canvas>` element
  - Runs the simulation to completion (auto-stop when alpha drops below threshold)
- [ ] Replace the stat cards with the canvas in `graph-view.component.ts`
- [ ] Keep stat counts as a small header bar above the canvas

**Deliverable:** You see your wiki as a graph. Nodes float into position. No interaction.

### Phase 2: Basic Interaction

**Goal:** Pan, zoom, drag nodes, hover tooltips.

- [ ] Implement mouse event handlers on the canvas:
  - `mousedown`/`mousemove`/`mouseup` for pan (empty space) and drag (node)
  - `wheel` for zoom (with `event.preventDefault()`)
  - `mousemove` for hover detection (nearest-node distance check)
- [ ] D3-zoom integration (`d3.zoom()`) for smooth zoom/pan with `transform` matrix
- [ ] Node pinning: when a node is dragged, set `fx`/`fy` on the simulation node to pin it
- [ ] Hover tooltip: show title + type + degree. Use an absolutely-positioned div (not canvas text) for crisp rendering.
- [ ] Cursor changes: `grab` on empty canvas, `grabbing` while panning, `pointer` on nodes

**Deliverable:** Interactive graph. Pan, zoom, drag, hover.

### Phase 3: Visual Encoding + Polish

**Goal:** Colors, sizes, labels, animations.

- [ ] Node color map by `page_type` (define in a shared constant — reuse across the app)
- [ ] Node radius proportional to `degree` with min/max clamp
- [ ] Edge colors by `edge_type` (subtle, semi-transparent)
- [ ] Labels: show `title` text next to nodes. Use canvas `fillText()`. Only render labels for nodes whose screen-space radius exceeds a threshold (i.e., only when zoomed in enough).
- [ ] Simulation warm-start: pre-compute a reasonable layout by initializing positions in a circle or grid, then let d3 settle from there. This prevents the "explosion" effect on first render.
- [ ] Smooth alpha reheating: when filters change or new nodes appear, reheat the simulation alpha so nodes animate into position

**Deliverable:** Visually appealing graph with proper color encoding and labels.

### Phase 4: Filtering & Search

**Goal:** Filter by page type, search and highlight nodes.

- [ ] Page type filter bar above the canvas (pill toggles for each type)
  - Clicking a pill toggles visibility of that type's nodes (and their incident edges)
  - "Select all" / "Deselect all" buttons
  - Filter state as a signal; filtered nodes/edges as computed signals
- [ ] Search bar: text input that filters nodes by ID or title substring match
  - Matching nodes get a highlight ring; non-matching nodes get dimmed (lower opacity)
  - Selecting a search result pans and zooms the view to center on that node
- [ ] Orphan toggle: highlight nodes with degree=0 in a warning color

**Deliverable:** Users can filter the graph to focus on what they care about.

### Phase 5: Local Graph + Click Navigation

**Goal:** Click to open pages, double-click for local graph, context menu.

- [ ] Single-click on a node: emit an event. `GraphViewComponent` navigates to Pages view with that page ID as query param, or opens a slide-out detail panel (TBD based on UX preference).
- [ ] Double-click on a node: switch to local graph mode:
  - Fetch `POST /api/graph/neighbors` with `depth: 2`
  - Replace full graph data with neighborhood data
  - Show the center node highlighted, neighbors arranged around it
  - Show a breadcrumb: "Global Graph > wiki:concepts:foo (local)"
- [ ] Right-click context menu (custom, not browser native):
  - "Open in Pages"
  - "Explore Neighbors" (same as double-click)
  - "Copy Page ID"
- [ ] Back button / "Show all" to return to global graph

**Deliverable:** The graph is fully navigable. It replaces the text-input neighbor explorer entirely.

### Phase 6: Polish & Performance

**Goal:** 60fps, responsive, good UX at all scales.

- [ ] Performance tuning for 500+ nodes:
  - Skip label rendering for nodes below a screen-size threshold
  - Batch canvas draw calls (minimize `stroke()`/`fill()` switches)
  - Throttle simulation ticks to 30fps for very large graphs (still renders at 60fps via interpolation)
  - If needed: WebGL renderer (pixi.js or regl) as a progressive enhancement
- [ ] Dark mode support: canvas background and node/edge colors adapt to `prefers-color-scheme` or app theme toggle
- [ ] Responsive canvas: resize with container using ResizeObserver
- [ ] Simulation presets in Settings: "Tight" (short edge length, strong gravity), "Spread" (long edges, weak gravity), "Fast" (high alpha decay, fewer ticks)
- [ ] Keyboard navigation (accessibility): Tab through nodes, arrow keys to move focus, Enter to select
- [ ] Empty state: if graph has zero nodes, show the same empty state pattern as other views ("No pages yet. Create your first page to see the graph.")

**Deliverable:** Production-quality graph view.

---

## 5. Integration Points

### 5.1 Replacement of Current Neighbor Explorer

The current text-input-based neighbor explorer (`nodeId` input + "Explore" button + neighbor card list) is **fully replaced** by the interactive graph. The workflow changes from:

```
Type page ID → Click Explore → Read card list
```
to:
```
Pan/zoom to find node → Hover to preview → Click to see details
     or
Search for page → Graph highlights it → Click to see details
     or
Double-click node → Local graph shows neighbors visually
```

The neighbor API endpoint is still used (extended) but only for local graph mode (Phase 5), not as the primary interaction.

### 5.2 Integration with Pages View

When a user clicks a node, the app should navigate to the Pages view with that page's content visible. Options:

- **Option A (simpler):** Navigate to `/pages?id=wiki:concepts:foo` — Pages view reads the query param and loads that page
- **Option B (rich):** Open a slide-out panel within the Graph view that shows the page content inline, without leaving the graph

**Recommendation:** Start with Option A (simpler, reuses existing Pages view). Option B can be added later as an enhancement.

### 5.3 Graph Data in NgRx Store

Per the production readiness spec (D-7: NgRx for state management), graph data should be stored in NgRx:

```typescript
// Store shape
interface GraphState {
  nodes: GraphNode[];
  edges: GraphEdge[];
  loading: boolean;
  error: string | null;
  filters: {
    pageTypes: string[];
    searchQuery: string;
    showOrphans: boolean;
  };
  selectedNodeId: string | null;
  mode: 'global' | 'local';
  localCenterId: string | null;
}
```

- `GraphViewComponent` dispatches `loadGraph()` action on init
- An effect calls `ApiService.getGraphFull()` and dispatches `loadGraphSuccess`
- Selectors provide `filteredNodes`, `filteredEdges` (derived from filters)
- The canvas directive receives filtered data via `@Input()`

### 5.4 Shared Color Palette

Node/page type colors should be defined in a shared constant used by:
- Graph canvas (node fill colors)
- Pages view (page type badge colors)
- Task board (task priority colors if they share a type system)
- Search results (page type indicators)

This ensures visual consistency across the app. Define in a shared file: `apps/wm-web/src/app/shared/page-type-colors.ts`

### 5.5 Backend: Graph Snapshot Consistency

The graph is stored as `ArcSwap<GraphSnapshot>` where each snapshot is a new `StableGraph` + `HashMap<String, NodeIndex>`. When the client fetches `/api/graph/full`, it gets a consistent snapshot because the handler loads the ArcSwap once and serializes from that single snapshot. No concern for partial/torn reads.

### 5.6 Existing Stats Cards

The current stats cards (Nodes/Edges count with dot grids) are not needed in the full graph view — the visual graph conveys much more information. However, a **compact summary bar** above the canvas showing "N nodes · M edges · K page types" is useful and can be kept. It can also show filtered counts: "Showing 42/156 nodes (filtered)".

---

## 6. Out of Scope (Explicitly)

These are explicitly **not** part of this spec and should be separate specs if pursued:

- **3D graph view** (three.js / force-graph-3d). Cool demo, not practical for knowledge work.
- **Graph analytics** (centrality metrics, community detection, PageRank). The graph view is for spatial browsing, not analysis.
- **Timeline/animation** of graph changes over time. Interesting but separate concern.
- **Collaborative graph editing** (multiple cursors, real-time sync). The graph is derived from wiki pages, not manually authored.
- **Export graph as image/PDF**. Nice-to-have but not core.
- **Graph-driven navigation** as the primary app shell (like Roam Research's graph). Our sidebar navigation stays.

---

## 7. Open Questions

These should be resolved before Phase 1 implementation begins:

1. **Canvas vs. WebGL for Phase 1?** Canvas is simpler and sufficient for <500 nodes. Should we start with Canvas and plan a WebGL migration path (e.g., abstract the renderer behind an interface), or commit to Canvas long-term? **Recommendation:** Start with Canvas. If we hit 1,000+ nodes, the renderer can be swapped without changing the simulation or interaction layer.

2. **d3-selection/d3-zoom dependency?** d3-force is standalone. Do we also pull in `d3-zoom` (for zoom/pan transforms) and `d3-selection` (for DOM utilities), or implement zoom/pan manually? **Recommendation:** Use `d3-zoom` — it's well-tested and handles touch/pinch natively. Avoid `d3-selection` — Angular's template system covers DOM manipulation.

3. **Node position persistence?** Should node positions be saved (e.g., in a backend endpoint or localStorage) so the graph looks the same each time you open it? Obsidian does not persist positions — the simulation re-runs each time. **Recommendation:** Skip persistence in Phase 1–5. If users complain, add localStorage-based position caching in Phase 6.

4. **Mobile support?** Touch gestures (pinch-to-zoom, tap-to-select) are supported by d3-zoom. But the graph experience on a phone screen with 500 nodes is cramped. **Recommendation:** Basic mobile support (pan/zoom works) but don't optimize the layout for mobile. The graph view is primarily a desktop experience.

---

## 8. References

- [d3-force documentation](https://d3js.org/d3-force)
- [d3-zoom documentation](https://d3js.org/d3-zoom)
- [Obsidian Graph View](https://help.obsidian.md/Plugins/Graph+view) — the UX benchmark
- [Graph Engine Spec](./graph-engine.md) — our backend graph architecture
- [Web UI Production Readiness](./web-ui-polish-production-readiness.md) — Angular/Spartan standards
- [ArcSwap Lock-Free Graph](../patterns/arc-swap-graph.md) — backend graph storage pattern
