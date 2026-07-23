---
title: WM Architecture
type: reference
tags:
- architecture
- system-design
- rust
- angular
- wasm
status: reviewed
relates_to:
  - {type: references, target: wiki:CONVENTIONS}
---

# WM Architecture

## High-Level System Design

wm uses a **single HTTP daemon** (`wm-server`) as the primary deployment target. The Angular frontend (`wm-web`) communicates with the server over HTTP. The CLI (`wm-cli`) runs engine code in-process for local operations. Graph rendering uses Canvas 2D with force-directed layout computed in-browser via WASM.

```
Client (Browser)               wm-server (Rust)           Storage
─────────────────              ──────────────────         ─────────
wm-web (Angular)                    │                      .wm/
  ├── Pages / Search ── HTTP ──►  wm-core                   wiki/*.md
  ├── Graph view                    ├── EngineState          memory/*.json
  │   ├── Canvas 2D render          │   ├── Graph (petgraph)  state/vectors.bin
  │   └── WASM layout (fjadra)      │   ├── BM25 indexes
  ├── Task board                    │   ├── Memory store
  └── Memory                        │   └── Tool registry
                                    ├── Page CRUD
                                    ├── Search router
                                    └── MCP tool surface
```

## Deployment Modes

| Mode | Binary | Transport | Use Case |
|------|--------|-----------|----------|
| HTTP daemon | `wm-server` | Axum REST :4090 | Web UI, remote access |
| MCP stdio | `wm-cli mcp` | JSON-RPC over stdio | AI agent integration |
| Direct CLI | `wm-cli` | In-process | Local operations, TUI |
| Web UI dev | `ng serve` | Dev proxy → :4090 | Frontend development |

The CLI never proxies through HTTP — it creates the engine in-process for offline operation and low latency.

## Crate Architecture

### Apps

| Crate | Role |
|-------|------|
| **wm-core** | Library crate — graph engine (petgraph), BM25 search, ONNX embeddings, page CRUD, task management, memory, MCP tool registry. All business logic. |
| **wm-cli** | Binary — clap CLI + Ratatui TUI. Creates engine in-process for local commands (`wm search`, `wm task`, etc.) and stdio MCP server (`wm mcp`). |
| **wm-server** | Binary — Axum HTTP daemon wrapping wm-core. Serves REST API on `:4090` for Angular frontend. Singleton engine process. |
| **wm-web** | Angular SPA — pages, search, graph visualization, task board, settings. Communicates with wm-server via HTTP. |

### Packages (Shared Libraries)

| Crate | Role |
|-------|------|
| **wm-engine** | Engine orchestration — coordinates graph rebuilds, index management, startup/shutdown lifecycle |
| **wm-search** | BM25 implementation with field-weighted scoring, RRF fusion, post-rerank heuristics. Code-aware tokenizer |
| **wm-embed** | ONNX embedding pipeline — vector generation, cosine similarity, vector persistence as flat binary |
| **wm-code-intel** | Code intelligence — AST-aware symbol search, dependency analysis via tree-sitter |
| **wm-lsp** | LSP integration for code-aware features |

### WASM Crates (Browser-Side Compute)

These follow the **fjadra profile**: cdylib + wasm-bindgen + serde, fs-free, tokio-free, rayon-optional, pure computation.

| Crate | Function |
|-------|----------|
| **fjadra-wasm** | Force-directed graph layout (simulation ticks in requestAnimationFrame) |
| **graph-algo-wasm** | Petgraph BFS path/neighbor/subgraph on fetched subgraphs |
| **bm25-rerank-wasm** | Client-side BM25 re-scoring of search results |
| **md-parse-wasm** | YAML frontmatter + markdown body extraction |

## Concurrency Model

All core data structures use **ArcSwap** for lock-free reads:

```rust
ArcSwap<(StableGraph<WikiPageMeta, EdgeType>, HashMap<String, NodeIndex>)>  // Graph
ArcSwap<Bm25Index>   // BM25 index
ArcSwap<VectorRegistry>  // Vector registry
```

**Pattern**: Build new version in background → atomic pointer swap via `ArcSwap::store`. Readers hold an `Arc` to the old snapshot and never block. Dirty-bit + directory mtime detects staleness for auto-rebuild.

## Search Pipeline

```
Query → BM25 (field weighted) ─┐
        Vector (if loaded) ────┤──→ RRF Fusion → Post-RRF Rerank → Results
        Memory BM25 (salience) ─┘   (rank merge)   (title density, exact match,
                                                    tag overlap, FSRS-6 recency)
```

## Graph Model

- **Storage**: `petgraph::StableGraph<WikiPageMeta, EdgeType>` — typed directed graph
- **Nodes**: All wiki pages (task, spec, concept, pattern, decision, howto, reference)
- **Edges**: 15+ built-in typed relationships (extends, implements, depends_on, part_of, references, etc.)
- **Traversal**: BFS for shortest-path + context assembly, DFS for full neighborhood
- **Edge declaration**: YAML frontmatter `relates_to` field

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| Single HTTP daemon | One engine state, no stale-data bugs, ~500MB saved per process |
| Canvas 2D (not WebGL) | Instant at current scale (~400 nodes), avoids GL complexity |
| MCP handlers direct (no proxy) | Proxy hid tools behind stale hardcoded lists |
| WASM only for pure compute | wm-core doesn't compile to wasm32 (tokio::fs, ort, rayon) |
| Sync writes (not async channels) | Single-user tool — async channels introduced races |
| NgRx for Angular state | Graph data, page state, UI state in one store |

## Non-negotiable

- No Node.js or Python backend services
- No external database (turso/SQLite is fine for local state)
- No third-party API dependencies for core functionality
- No `any` types in Angular (strict mode)
- No `#[allow(dead_code)]`

## References

- @wiki/conventions:enterprise-grade — Scale targets and locked decisions
- @wiki/concepts:graph-architecture — Graph model internals
- @wiki/concepts:memory-system — Memory layer design
- @wiki/conventions:wm-conventions — Code and project conventions
- @wiki/decisions:cli-direct-execution-not-http-proxy
- @wiki/decisions:mcp-direct-handlers-over-proxy