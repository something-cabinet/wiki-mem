---
title: Enterprise-Grade Architecture
type: concept
tags: [architecture, enterprise, performance, scale, locked]
status: reviewed
---

# Enterprise-Grade Architecture

This project targets enterprise deployments with large-scale knowledge graphs, not small personal wikis.

## Scale Requirements

| Metric | Target |
|--------|--------|
| Graph nodes | 100,000+ |
| Graph edges | 500,000+ |
| Pages | 50,000+ |
| Graph render | 60fps at 100k nodes |

## Locked Decisions

### D1: Tauri v2 primary, all-in

**Decision:** Tauri v2 desktop app is the primary deployment. The HTTP server (`wm-server`) is removed entirely.

**Rationale:** Enterprise users get a native desktop app with direct Rust IPC, system tray, native menus, background tasks, and file system access. No HTTP overhead, no port management, no proxy config.

**What stays:**
- `wm-core` — Rust library, imported directly by Tauri backend
- `wm-cli mcp` — stdio MCP server for OpenCode AI agent integration
- `wm-web` — Angular frontend, bundled statically by Tauri

**What goes:**
- `wm-server` — Axum HTTP server deleted (routes become Tauri commands)

**Tauri IPC replaces:**
| wm-server route | Tauri equivalent |
|----------------|-----------------|
| `POST /api/tools` | `#[tauri::command]` dispatch |
| `POST /api/initial` | `#[tauri::command]` |
| `POST /api/search` | `#[tauri::command]` |
| `POST /api/pages/*` | 5 commands (CRUD) |
| `POST /api/tasks/board` | `#[tauri::command]` |
| `POST /api/graph/*` | Stats, neighbors, full graph |
| `POST /api/memory/list` | `#[tauri::command]` |
| `GET /api/events` (SSE) | Tauri events `emit()`/`listen()` |
| SPA static serving | Tauri native file serving |
| `graph_rebuild_loop` | Tauri `setup()` via `tokio::spawn` |

### D2: Graph = Rust layout + WebGL rendering

**Decision:** The graph view uses Rust-native layout (computed in Tauri backend, multi-threaded, GPU-accelerated via `wgpu`) and renders via WebGL (not Canvas 2D).

**Rationale:** 100k+ nodes require WebGL for batch rendering (~0.01ms per draw call) and GPU-accelerated layout compute (Barnes-Hut O(n log n) via compute shaders). d3-force + Canvas 2D is insufficient at this scale — they fail around 5k-10k nodes.

**Implementation:**
- Layout runs in Rust (Tauri IPC) using `fjadra` or custom `wgpu` compute shaders
- Rendering is WebGL instanced drawing (one draw call per node type)
- Binary data transfer (Float32Array) between Rust and WebGL, not JSON
- Level-of-detail system: Rust computes cluster hierarchy; WebGL shows micro/macro view based on zoom
- Prototype phases 1-2 may use Canvas 2D + d3-force for rapid iteration, but the architecture must be abstracted for the WebGL + Rust swap in phase 3+

### D3: NgRx for state management

**Decision:** Graph data, page state, and UI state live in NgRx.

### D4: No WASM

**Decision:** WASM compilation is unnecessary. Tauri gives us native Rust execution in the same process as the frontend. WASM would add build complexity (wasm-pack, web-sys bindings) without benefit.

### D5: No external services

**Decision:** No Node.js, Python, or external API dependencies for core functionality. SQLite via turso is acceptable for local state.

## Graph Rendering Architecture

```
Angular (webview)                    Tauri Rust Backend
─────────────────                    ──────────────────
                                     wm-core (imported)
                                       ├── EngineState (graph, BM25, etc.)
                                       ├── Page CRUD
                                       ├── Search
                                       ├── Task board
                                       └── Memory
                                           
GraphViewComponent                       │
  ├── WebGL canvas                       │ invoke('compute_layout')
  ├── Filter toolbar                    ←┤ positions → Float32Array
  ├── Search bar                         │
  └── Slide-out panel                    │ invoke('get_page')
                                       ←│
```

## Non-negotiable

- No Node.js backend services
- No Python services
- No external database (turso/SQLite is fine for local state)
- No third-party API dependencies for core functionality
- No `any` types in Angular (strict mode)

## References

- [Tauri v2 docs](https://v2.tauri.app)
- [wgpu compute shaders](https://wgpu.rs)
- [Obsidian Graph View](https://help.obsidian.md/Plugins/Graph+view)
- [Graph Spec](./specs/obsidian-graph-view.md)
