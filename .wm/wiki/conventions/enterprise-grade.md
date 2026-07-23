---
title: Enterprise-Grade Architecture
type: concept
tags: [architecture, enterprise, performance, scale, locked]
status: reviewed
relates_to:
  - {type: supersedes, target: wiki:specs:single-http-server}
  - {type: references, target: wiki:specs:stress-scale-tests}
  - {type: references, target: wiki:memory:tauri-async-blocking-simulation-loops}
  - {type: references, target: wiki:tasks:review-blocking-async-fjadra-layout}
---

# Enterprise-Grade Architecture

This project targets enterprise deployments with large-scale knowledge graphs, not small personal wikis.

## Scale Requirements

| Metric | Current | Target |
|--------|---------|--------|
| Graph nodes | 331 | 10,000+ |
| Graph edges | 25 | 50,000+ |
| Pages | ~300 | 5,000+ |
| Graph render | Instant at 331 nodes | 60fps at 10k nodes (stretch) |

> **Scale target:** 10k nodes comfortable. Layout is in-browser WASM (fjadra). Rendering is Canvas 2D; WebGL is aspirational for 10k+ node counts.

## Architecture Summary

The project uses a **single HTTP daemon** (`wm-server`) as the primary deployment target. The Angular frontend (`wm-web`) communicates with the server over HTTP. Graph rendering uses Canvas 2D with force-directed layout computed in-browser via WASM (fjadra).

```
Client (Browser)               wm-server (Rust)
─────────────────              ──────────────────
wm-web (Angular)                   │
  ├── Pages / Search ── HTTP ──►  wm-core
  ├── Graph view                   ├── EngineState (graph, BM25, etc.)
  │   ├── Canvas 2D render         ├── Page CRUD
  │   └── WASM layout (fjadra)     ├── Search
  └── Task board                   ├── Task board
                                   └── Memory
```

## Locked Decisions

### D1: [OBSOLETE — Tauri v2 primary, all-in]

This decision is superseded. The project now uses a single HTTP daemon (`wm-server`) as the primary deployment target, not Tauri v2. The Angular frontend communicates over HTTP. See [@wiki/specs/single-http-server](../specs/single-http-server.md) for the current architecture.

### D2: [UPDATED] Canvas 2D + Browser WASM Layout

**Decision (current):** Graph rendering uses Canvas 2D. Layout runs in-browser via fjadra WASM, not server-side.

**Rationale:** Canvas 2D with ~400 nodes is instant and avoids WebGL complexity (shader management, texture atlases, GL context loss). WASM layout eliminates HTTP round-trip per frame and enables interactive tuning (spacing slider re-runs simulation in-browser). For 10k+ nodes, WebGL remains aspirational but is not needed at current scale.

**Implementation (current):**
- Layout: fjadra compiled to WASM via wasm-pack, runs in `requestAnimationFrame` tick loop (3×15 iterations per frame)
- Rendering: Canvas 2D (ctx.arc for nodes, ctx.lineTo/bezierCurveTo for edges)
- Spacing slider re-creates WASM simulation with new parameters
- Lazy-loaded: WASM module imported dynamically only when graph view mounts

**Future:** If node counts exceed 10k and Canvas 2D drops below 30fps, evaluate WebGL or PixiJS. WASM layout stays regardless.

### D3: NgRx for state management

**Decision:** Graph data, page state, and UI state live in NgRx.

### D4: [UPDATED] Selective WASM for Browser-Side Compute

**Decision:** WASM is used for pure-compute, stateless operations that benefit from in-browser execution. The Rust backend (`wm-server`) handles all filesystem I/O and stateful operations.

**Profile (fjadra pattern):** A WASM crate must be fs-free, tokio-free, rayon-optional, and pure computation (data in → compute → data out). Current WASM crates:
- `fjadra-wasm` — force-directed graph layout
- `graph-algo-wasm` — petgraph BFS path/neighbor/subgraph on fetched subgraphs
- `bm25-rerank-wasm` — client-side BM25 re-scoring
- `md-parse-wasm` — YAML frontmatter + markdown body extraction

**Build:** `wasm-pack build --target web`, output in Angular assets as lazy-loaded ES module chunks.

### D5: No external services

**Decision:** No Node.js, Python, or external API dependencies for core functionality. SQLite via turso is acceptable for local state.

## Non-negotiable

- No Node.js backend services
- No Python services
- No external database (turso/SQLite is fine for local state)
- No third-party API dependencies for core functionality
- No `any` types in Angular (strict mode)

## References

- [Single HTTP Server Spec](../specs/single-http-server.md)
- [wgpu compute shaders](https://wgpu.rs)
- [Obsidian Graph View](https://help.obsidian.md/Plugins/Graph+view)
- [Graph Spec](./specs/obsidian-graph-view.md)
