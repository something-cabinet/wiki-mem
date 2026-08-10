---
title: WM Architecture
type: core
tags:
- architecture
- system-design
- rust
- angular
- wasm
status: reviewed
relates_to:
  - {type: references, target: wiki:core:CONVENTIONS}
---

---
title: WM Architecture
type: core
tags:
- architecture
- system-design
- rust
- angular
- wasm
status: reviewed
relates_to:
  - {type: references, target: wiki:core:CONVENTIONS}
---

# WM Architecture

## High-Level System Design

wm uses a **single HTTP daemon** (`wm-server`) as the primary deployment target. The Angular frontend (`wm-web`) communicates with the server over HTTP. The CLI (`wm-cli`) is a **thin HTTP client** for most commands (search/page/graph/source/task/lint/validate/index/time), spawning the daemon if absent; only init/setup/upgrade/migrate-memory run in-process. MCP is a **stdio→HTTP proxy** in `wm-cli` targeting the daemon's privileged `/api/mcp/*` channel. Graph rendering uses Canvas 2D with force-directed layout computed in-browser via WASM.

```
Client (Browser)               wm-server (Rust)           Storage
─────────────────              ──────────────────         ─────────
wm-web (Angular)                    │                      .wm/
  ├── Pages / Search ── HTTP ──►  wm-core                   wiki/*.md
  ├── Graph view                    ├── EngineState          wiki/memory/*.md
  │   ├── Canvas 2D render          │   ├── Graph (petgraph)  state/vectors.db
  │   └── WASM layout (fjadra)      │   ├── BM25 indexes
  ├── Task board                    │   ├── Memory store
  └── Memory                        │   └── Tool registry
                                    ├── Page CRUD
                                    ├── Search router
                                    └── MCP tool surface
wm-cli mcp (stdio→HTTP proxy) ──►   POST /api/mcp/tools/* (mcp-token)
wm-cli commands ────────────────►   HTTP :4090 (web-token / mcp-token)
```

## Deployment Modes

| Mode | Binary | Transport | Use Case |
|------|--------|-----------|----------|
| HTTP daemon | `wm-server` | Axum REST :4090 | Web UI, remote access, all tool dispatch |
| MCP stdio | `wm-cli mcp` | stdio JSON-RPC → HTTP proxy | AI agent integration |
| CLI commands | `wm-cli` | HTTP calls to daemon | search/page/graph/task/lint/validate/index/time |
| Local-only | `wm-cli` | In-process | init, setup, upgrade, migrate-memory |
| Web UI dev | `ng serve` | Dev proxy → :4090 | Frontend development |

The CLI routes commands through the daemon over HTTP (spawning it if absent); it does NOT create a second in-process `EngineState` for the migrated command set. A few filesystem/install commands stay local by design.

## Crate Architecture

### Apps

| Crate | Role |
|-------|------|
| **wm-core** | Library crate — graph engine (petgraph), BM25 search, ONNX embeddings, page CRUD, task management, memory, MCP tool registry. All business logic. rmcp is optional (enabled by transport owners). |
| **wm-cli** | Binary — clap CLI + Ratatui TUI + MCP proxy (`mcp_proxy.rs`, stdio→HTTP via ureq). CLI commands call the daemon over HTTP; init/setup/upgrade/migrate-memory run in-process. |
| **wm-server** | Binary — Axum HTTP daemon wrapping wm-core. Serves REST API on `:4090` for Angular, web-token-gated read-only web surface, privileged `/api/mcp/*` channel (mcp-token), SPA serving, `.wm/server.json` singleton + discovery. |
| **wm-web** | Angular SPA — pages, search, graph visualization, task board, settings. Communicates with wm-server via HTTP. |

### Packages (Shared Libraries)

| Crate | Role |
|-------|------|
| **wm-constants** | Zero-dependency shared constants — magic values used in 3+ crates (`.wm`, `wiki`, `state`, skip dirs, file names, default port/limits). Sits at the bottom of the dep graph. |
| **wm-engine** | Engine orchestration — coordinates graph rebuilds, index management, startup/shutdown lifecycle |
| **wm-search** | BM25 implementation with field-weighted scoring, RRF fusion, post-rerank heuristics. Code-aware tokenizer |
| **wm-embed** | ONNX embedding pipeline — vector generation, cosine similarity, vector persistence (turso), version/chunking metadata tracking, session-per-thread |
| **wm-code-intel** | Code intelligence — AST-aware symbol search, dependency analysis via tree-sitter |
| **wm-lsp** | LSP integration for code-aware features |

### WASM Crates (Browser-Side Compute)

These follow the **fjadra profile**: cdylib + wasm-bindgen + serde, fs-free, tokio-free, rayon-optional, pure computation.

| Crate | Function |
|-------|----------|
| **fjadra-wasm** | Force-directed graph layout (simulation ticks in requestAnimationFrame, configurable charge strength) |
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

**Write-path freshness**: the in-memory graph snapshot is refreshed synchronously after every page write (`graph::handle_file_change` in create/update/delete) — the daemon runs no file watcher, so writers refresh derived state themselves rather than relying on a watcher/rebuild.

## Search Pipeline

```
Query → BM25 (field weighted) ─┐
        Vector (if loaded) ────┤──→ RRF Fusion → Post-RRF Rerank → Results
        Memory BM25 (salience) ─┘   (rank merge)   (title density, exact match,
                                                    tag overlap, FSRS-6 recency)
```

## Graph Model

- **Storage**: `petgraph::StableGraph<WikiPageMeta, EdgeType>` — typed directed graph
- **Nodes**: All wiki pages (task, spec, concept, pattern, decision, howto, reference, core)
- **Edges**: 9 built-in typed relationships (extends, implements, depends_on, part_of, references, example_of, supersedes, answers, relates_to)
- **Traversal**: BFS for shortest-path + context assembly, DFS for full neighborhood
- **Edge declaration**: YAML frontmatter `relates_to` field

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| Single HTTP daemon | One engine state, no stale-data bugs, ~500MB saved per process |
| Canvas 2D (not WebGL) | Instant at current scale (~400 nodes), avoids GL complexity |
| MCP = stdio→HTTP proxy | Single writer; privileged `/api/mcp/*` channel + separate mcp-token; dynamic tools/list from registry (see decision mcp-proxy-privileged-channel-token-split) |
| WASM only for pure compute | wm-core doesn't compile to wasm32 (tokio::fs, ort, rayon) |
| Sync writes (not async channels) | Single-user tool — async channels introduced races |
| CLI commands over HTTP | Thin client shares the daemon; init/setup/upgrade/migrate-memory stay local |

## Non-negotiable

- No Node.js or Python backend services
- No external database (turso/SQLite is fine for local state)
- No third-party API dependencies for core functionality
- No `any` types in Angular (strict mode)
- No `#[allow(dead_code)]`

## References

- @wiki/core:enterprise-grade — Scale targets and locked decisions
- @wiki/concepts:graph-architecture — Graph model internals
- @wiki/concepts:memory-system — Memory layer design
- @wiki/core:conventions — Code and project conventions
- @wiki/decisions:mcp-proxy-privileged-channel-token-split — MCP proxy architecture + token split