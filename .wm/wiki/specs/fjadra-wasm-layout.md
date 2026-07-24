---
id: wiki:specs:fjadra-wasm-layout
title: fjadra WASM Force Layout
type: spec
status: approved
tags: [spec, graph, wasm, layout, fjadra]
---
id: wiki:specs:fjadra-wasm-layout


## Overview

Replace the current server-side force layout (`POST /api/graph/layout`) with a browser-side fjadra WASM module. The layout computation moves from Rust server to Rust→WASM running in the Angular frontend, eliminating the HTTP round-trip for layout and enabling progressive position streaming without SSE.

## Locked Decisions

- D1: **Replace server layout** — fjadra WASM in browser, server layout removed
- D2: **New `packages/fjadra-wasm` crate** — standalone wasm-bindgen wrapper
- D3: **Progressive API** — `create()` returns a handle, `tick(n)` advances the simulation, `getPositions()` returns current positions

## Requirements

### Functional Requirements

**FR-1: fjadra WASM Wrapper Crate**
Create `packages/fjadra-wasm/` with:
- `Cargo.toml` depending on `fjadra = "0.2.1"` and `wasm-bindgen = "0.2"`
- `crate-type = ["cdylib"]` for WASM output
- wasm-bindgen wrapper exposing:

```rust
#[wasm_bindgen]
pub struct SimulationHandle { ... }

#[wasm_bindgen]
impl SimulationHandle {
    pub fn create(node_count: usize, center_x: f64, center_y: f64, spread: f64) -> SimulationHandle;
    pub fn add_link(&mut self, source: usize, target: usize, distance: f64, strength: f64);
    pub fn tick(&mut self, iterations: usize);
    pub fn get_positions(&self) -> Vec<f64>;  // flat [x0, y0, x1, y1, ...]
    pub fn is_finished(&self) -> bool;
}
```

**FR-2: Build Script**
- Add `wasm-pack build -p fjadra-wasm --target web` to build pipeline
- Output `.wasm` + JS glue to `apps/wm-web/src/assets/wasm/` (or similar)
- Add a npm script: `npm run build:wasm`

**FR-3: Angular Integration**
- Load WASM module in graph-view component on init
- Replace `startLayout()` HTTP POST with:
  1. Create `SimulationHandle` with node count + viewport dimensions
  2. Add link forces for each edge
  3. Loop: `tick(10)` → `getPositions()` → `applyPositions()` → requestAnimationFrame
  4. Stop when `is_finished()` or max ticks reached

**FR-4: Remove Server Layout**
- Remove `POST /api/graph/layout` route
- Remove the custom force layout code from `layout.rs`
- The SSE streaming endpoint can stay as a stub or be removed

**FR-5: Spacing Slider Wiring**
- `link_distance` from the slider passed directly to `add_link()` on WASM creation (no HTTP needed)

### Non-Functional Requirements

- NFR-1: WASM module loads within 500ms (cached after first load)
- NFR-2: Graph stays interactive during WASM layout computation (tick yields via setTimeout/rAF)
- NFR-3: No regression in graph node/edge rendering

## Acceptance Criteria

- [ ] AC-1: `wasm-pack build` produces a working `.wasm` + JS glue
- [ ] AC-2: Graph layout computes in the browser with correct positions
- [ ] AC-3: Progressive position updates during simulation (not a blocking freeze)
- [ ] AC-4: No server POST to `/api/graph/layout` after migration
- [ ] AC-5: Spacing slider value affects layout
- [ ] AC-6: Server-side layout code removed from `layout.rs`
- [ ] AC-7: Angular build succeeds with WASM integration

## Scenarios

### Scenario 1: User opens graph view
**Given** the graph has 400 nodes with edges
**When** the user navigates to the graph view
**Then** WASM module loads, simulation starts immediately, positions stream progressively as they converge, no HTTP layout call is made

### Scenario 2: User adjusts spacing
**Given** the graph is rendered
**When** the user moves the spacing slider
**Then** the simulation restarts with the new link_distance value

### Scenario 3: WASM fails to load
**Given** the browser doesn't support WASM or the module fails
**Then** the graph shows an error state with a retry button (not silent failure)

## Technical Notes

### Build Setup
```bash
wasm-pack build packages/fjadra-wasm --target web --out-dir ../../apps/wm-web/src/assets/wasm
```

### Angular Loading Pattern
```typescript
import init, { SimulationHandle } from '../assets/wasm/fjadra_wasm';

async function runLayout(nodes, edges, width, height, linkDistance) {
  await init(); // load .wasm once
  const sim = SimulationHandle.create(nodes.length, width / 2, height / 2, Math.min(width, height) * 0.3);
  for (const { source, target } of edges) {
    sim.add_link(source, target, linkDistance, 0.3);
  }
  while (!sim.is_finished()) {
    sim.tick(10);
    const positions = sim.get_positions(); // flat Vec<f64>
    applyPositions(positions);
    await new Promise(r => requestAnimationFrame(r));
  }
}
```

### Dependencies to Add
- `wasm-pack` (cargo install or npm)
- `packages/fjadra-wasm/Cargo.toml`: `fjadra = "0.2.1"`, `wasm-bindgen = "0.2"`

### Files to Remove
- Server layout implementation in `apps/wm-server/src/routes/layout.rs` (the `start_layout` function)
- The force layout custom code

## Open Questions

- [ ] Should we keep the `stream_events` SSE endpoint for backward compatibility?
- [ ] Where exactly should the WASM build output land? `apps/wm-web/src/assets/wasm/` or a separate dir?
- [ ] Add `wasm-pack` to CI or document as prerequisite?
