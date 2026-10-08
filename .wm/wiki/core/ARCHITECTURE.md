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

schema_version: 1
state: |-
  # WM Architecture

  ## High-Level System Design

  wm-core is a library. `wm-cli` commands dispatch **in-process** against the tool registry, and `wm-cli mcp` serves rmcp stdio **in-process** (no daemon, no tokens). There is no HTTP surface and no bundled web UI.

  ```
  wm-cli mcp (rmcp stdio, in-process)   no daemon / no tokens
  wm-cli commands (in-process wm_core)  no daemon / no tokens
          │
          ▼
       wm-core (library)
         ├── EngineState
         │   ├── Graph (petgraph)
         │   ├── BM25 indexes
         │   ├── Memory store
         │   └── Tool registry
         ├── Page CRUD
         ├── Search router
         └── MCP tool surface
          │
          ▼
        .wm/
          wiki/*.md
          wiki/memory/*.md
          state/vectors.db
  ```

  ## Deployment Modes

  | Mode | Binary | Transport | Use Case |
  |------|--------|-----------|----------|
  | MCP stdio | `wm-cli mcp` | rmcp stdio, in-process registry | AI agent integration |
  | CLI commands | `wm-cli` | In-process wm_core dispatch | search/page/graph/task/lint/validate/index/time |
  | Local-only | `wm-cli` | In-process | init, setup, upgrade, migrate-memory |

  The CLI and MCP dispatch against an in-process engine (one `EngineState` per invocation). Filesystem/install commands (init/setup/upgrade/migrate-memory) stay local by design.

  ## Crate Architecture

  ### Apps

  | Crate | Role |
  |-------|------|
  | **wm-core** | Library crate — graph engine (petgraph), BM25 search, ONNX embeddings, page CRUD, task management, memory, MCP tool registry. All business logic. rmcp is optional (enabled by the in-process MCP server). |
  | **wm-cli** | Binary — clap CLI + in-process MCP (`mcp_server.rs`, rmcp stdio). CLI commands dispatch in-process against wm_core's registry; init/setup/upgrade/migrate-memory run in-process. |

  ### Packages (Shared Libraries)

  | Crate | Role |
  |-------|------|
  | **wm-constants** | Zero-dependency shared constants — magic values used in 3+ crates (`.wm`, `wiki`, `state`, skip dirs, file names, default port/limits). Sits at the bottom of the dep graph. |
  | **wm-engine** | Engine orchestration — coordinates graph rebuilds, index management, startup/shutdown lifecycle |
  | **wm-search** | BM25 implementation with field-weighted scoring, RRF fusion, post-rerank heuristics. Code-aware tokenizer |
  | **wm-embed** | ONNX embedding pipeline — vector generation, cosine similarity, vector persistence (turso), version/chunking metadata tracking, session-per-thread |
  | **wm-code-intel** | Code intelligence — AST-aware symbol search, dependency analysis via tree-sitter |
  | **wm-lsp** | LSP integration for code-aware features |

  ## Concurrency Model

  All core data structures use **ArcSwap** for lock-free reads:

  ```rust
  ArcSwap<(StableGraph<WikiPageMeta, EdgeType>, HashMap<String, NodeIndex>)>  // Graph
  ArcSwap<Bm25Index>   // BM25 index
  ArcSwap<VectorRegistry>  // Vector registry
  ```

  **Pattern**: Build new version in background → atomic pointer swap via `ArcSwap::store`. Readers hold an `Arc` to the old snapshot and never block. Dirty-bit + directory mtime detects staleness for auto-rebuild.

  **Write-path freshness**: the in-memory graph snapshot is refreshed synchronously after every page write (`graph::handle_file_change` in create/update/delete). Long-lived `wm mcp` sessions additionally run a notify file watcher (`MainEngine::with_root`) so external disk edits refresh automatically; one-shot CLI invocations rely on the synchronous refresh.

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
  | In-process dispatch for CLI + MCP | One engine per process, no daemon dependency, no stale-data bugs, no transport surface to secure |
  | Sync writes (not async channels) | Single-user tool — async channels introduced races |
  | MCP in-process (not HTTP proxy) | `wm-cli mcp` owns the registry directly; no daemon, no tokens, no readiness races (see decision cli-direct-execution-not-http-proxy) |

  ## Non-negotiable

  - No Node.js or Python backend services
  - No external database (turso/SQLite is fine for local state)
  - No third-party API dependencies for core functionality
  - No `#[allow(dead_code)]`

  ## References

  - @wiki/core:enterprise-grade — Scale targets and locked decisions
  - @wiki/concepts:graph-architecture — Graph model internals
  - @wiki/concepts:memory-system — Memory layer design
  - @wiki/core:conventions — Code and project conventions
questions:
  - id: kind
    type: choice
    instructions: What kind of core document is this?
    options:
    - conventions
    - architecture
    - patterns
    - glossary
    - reference
  - id: audience
    type: choice
    instructions: Who is this core document written for?
    options:
    - human
    - agent
    - both
  - id: mutability
    type: choice
    instructions: How mutable is this core document?
    options:
    - stable
    - evolving
    - frozen
answers: {}
