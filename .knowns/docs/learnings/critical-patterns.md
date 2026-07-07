---
title: Critical Patterns
description: Promoted learnings that save the most time. Read at session start.
createdAt: '2026-06-16T04:28:11.939Z'
updatedAt: '2026-07-07T10:35:05.402Z'
tags:
  - learning
  - critical
---

# Critical Patterns

Promoted learnings from completed work. Read this at the start of every session via wm-init. These are lessons that cost the most to learn and save the most by knowing.

---

## [2026-06-16] ArcSwap Co-Swap for Graph State
**Category:** decision
**Source:** @task-awotvr, @task-r8n30s
**Tags:** [graph, concurrency, arc-swap]

When you have a data structure (like a graph) that's rebuilt entirely on mutation, and you have concurrent readers: DON'T use RwLock. Build the new version in background, then atomically swap via ArcSwap. Readers hold an Arc to the old snapshot and never block. The graph and its id_index must be co-swapped atomically to prevent dangling NodeIndex references.

**Full entry:** @doc/learnings/learning-wiki-mem-graph-architecture

---

## [2026-06-16] SQLite Bundled When Already Using ONNX — SUPERSEDED
**Category:** decision (overridden)
**Source:** @task-g2gckv
**Tags:** [storage, sqlite, vectors, superseded]

**Superseded 2026-07-04:** Use flat binary `vectors.bin` instead. Vectors are derivable from wiki pages (no data loss on corruption). Adding `rusqlite` bundled costs ~30s compile time and adds another C dependency to audit. The vectors.bin format at 200 lines is simpler, avoids schema migrations, and the mmap approach (memmap2) keeps memory usage low. WAL crash safety doesn't help when the source of truth is elsewhere.

**Original:** If ort (ONNX Runtime) is already in your crate stack, rusqlite bundled adds zero marginal C dependency burden. Use SQLite for vector persistence — WAL mode crash safety, incremental upserts, schema migrations. Don't write a custom flat binary format just to avoid "a database." The 4ms startup overhead vs flat binary is negligible.

**Full entry:** @doc/learnings/learning-wiki-mem-graph-architecture

---

## [2026-06-16] Prefix MCP Tools to Avoid Collisions
**Category:** decision
**Source:** @task-j4tx6c
**Tags:** [mcp, naming]

Host apps (OpenCode, Kiro) may have built-in tools with generic names like `code`, `search`, `page`. Always prefix your MCP tools with a project-specific namespace (`wm_`, `gh_`, etc.). The prefix is explicit in the tool registration — don't rely on the client to namespace.

**Full entry:** @doc/learnings/learning-wiki-mem-graph-architecture

---

## [2026-06-16] CLI Is Bootstrap, MCP Is the Product — SUPERSEDED
**Category:** decision (overridden)
**Source:** @task-j4tx6c
**Tags:** [cli, mcp, architecture, superseded]

**Superseded 2026-07-04:** The CLI IS the product — TUI-first with Ratatui. Inspired by Knowns (Bubble Tea), the CLI provides instant, keyboard-driven access to the full wiki: search, browse, graph, tasks, time tracking, lint, source management. Interactive TUI when running in a terminal (auto-detected), plain text or JSON when piped. MCP remains the integration protocol for AI agents (OpenCode, Claude Code, Codex), but the CLI is the primary human interface.

**Original:** Only build enough CLI to bootstrap the system (init, serve, version). Everything else goes through MCP. A full CLI creates maintenance burden and delays MCP adoption. The CLI exists for testing and one-time setup, not as a primary interface.

**Why superseded:** The Knowns/Bubble Tea model proved that a TUI-first CLI is the best developer experience. AI agents use MCP natively, but humans reach for the terminal. A rich CLI with Ratatui TUI gives instant feedback (no browser, no server) while the web UI (SvelteKit/wm-ui) is optional for rich visualization. The CLI and MCP share the same engine — tools.rs handlers are called by both, so there's no duplication or drift.

**Full entry:** @doc/learnings/learning-wiki-mem-graph-architecture

---

## [2026-07-06] MCP Bridge Pattern for Web UIs
**Category:** pattern
**Source:** @task-umpd47, @task-s2ff4x
**Tags:** [mcp, web-ui, architecture]

When building a web UI on top of an MCP-based Rust backend: use a thin Node.js bridge that spawns the Rust process as a child and pipes JSON-RPC 2.0 over stdin/stdout. Don't add an HTTP server crate to the Rust code. The bridge (`wm-bridge.ts`) is ~50 lines, handles `sendRequest()` → write to stdin → read from stdout. SvelteKit API routes delegate to this bridge. This keeps the Rust backend pure MCP and the protocol testable independently.

**Full entry:** @doc/learnings/learning-post-build-quality-pass-spec-alignment-tui-mcp-integration

---

## [2026-07-06] Fixing Pre-Release Crate API Drift
**Category:** failure
**Source:** @task-kq0kld
**Tags:** [dependencies, onnx, embed]

When `ort 2.0.0-rc.12` API drifts from your code (13 compile errors): (1) Check the crate's actual `src/` files, not its published docs — docs lag behind rc releases. (2) Try the simplest data format first — `(Vec<i64>, Vec<i64>)` tuples for tensor creation, before fighting `ndarray` + `OwnedTensorArrayData` trait implementations. (3) Build `--features embed` in CI to catch drift early. The final fix used `Tensor::from_array((shape, data))` with plain Vecs — simpler than ndarray and fully compatible.

**Full entry:** @doc/learnings/learning-post-build-quality-pass-spec-alignment-tui-mcp-integration

---

## [2026-07-07] Knowns Core = Memory Layer, Not Spec System
**Category:** decision
**Source:** @task-29fizw
**Tags:** [memory, knowns, architecture, openspec]

OpenSpec (@fission-ai/openspec, 59k stars) is a dedicated spec system with change folders and lifecycle. Knowns/WM specs are a thin technique (Socratic exploration + doc template). The real value is the **memory substrate**: typed graph edges, semantic retrieval, context assembly, cross-references, AC tracking. WM should double down on the memory layer — not compete with OpenSpec on spec lifecycle management.

**Full entry:** @doc/learnings/learning-knowns-memory-layer-not-a-spec-system

---

## [2026-07-07] Per-Type BM25 + RRF + FSRS Recency
**Category:** pattern
**Source:** @spec/cross-entity-hybrid-search
**Tags:** [search, ranking, fsrs, rrf]

When searching across heterogeneous entity types (pages + memory), use per-type BM25 indexes merged via RRF (not unified index). Tasks stay in the page index with FSRS-6 recency boost — not a separate task index. Memory gets flat text context in retrieve, not graph BFS. Recency model defaults to FSRS-6 (hardcoded 21 params, only stability configurable).

**Full entry:** @doc/learnings/learning-cross-entity-search-per-type-bm25-fsrs-recency-debounced-indexscheduler

---

## [2026-07-07] Async Write Channel Race — Use Sync Writes
**Category:** failure
**Source:** @doc/learnings/learning-e2e-test-infrastructure-sync-write-fix
**Tags:** [write-channel, async, tokio, race]

For single-user local tools using tokio, do NOT route file writes through async channels. The return-before-flush semantic causes races with any operation that reads from disk. Use `std::fs::write()` directly. If you must use async writes, add a flush barrier with proper synchronization that doesn't deadlock the tokio runtime (use `spawn_blocking`, not blocking on worker threads).

**Full entry:** @doc/learnings/learning-e2e-test-infrastructure-sync-write-fix

---

## [2026-07-07] tools.rs Split Into Domain Modules
**Category:** pattern
**Source:** @spec/architectural-refactors-toolsrs-split-dependency-inversion-extraction
**Tags:** [architecture, refactor, mcp, tools]

When `mcp/tools.rs` exceeds ~1000 lines, split it into per-domain modules under `mcp/tools/`. Each domain module has a `pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>)` function. The parent `tools.rs` becomes a ~30-line delegator calling each module via `pub mod search; pub use search::*;`. Domain names match tool prefixes (`search.rs` → `wm_search.*`, `page.rs` → `wm_page.*`). This kept handler code discoverable, prevented merge conflicts, and reduced cognitive load. The old tools.rs was 1769 lines across 14 domains — now each module is 100-250 lines.

**Full entry:** @doc/learnings/learning-gehenna-app-cross-project-patterns-cdd-error-chains-svelte-5

---

## [2026-07-07] ToolError Typed Error Chaining
**Category:** pattern
**Source:** @doc/learnings/learning-gehenna-app-cross-project-patterns-cdd-error-chains-svelte-5
**Tags:** [error-handling, rust, toolerror]

ToolError should carry `source: Option<Box<dyn StdError>>` to preserve the full error context chain. Use specific constructors instead of generic `internal()`: `io_error(op, path, err)` for I/O failures, `serde_error(op, err)` for serialization, `lock_poisoned(resource)` for mutex poison. Removed PartialEq+Eq derives (incompatible with error chaining). The `Display` impl shows the chain; `Error::source()` returns the underlying error. This pattern comes from gehenna-app's `RepoError` which wraps `sea_orm::DbErr` with `#[source]`.

**Full entry:** @doc/learnings/learning-gehenna-app-cross-project-patterns-cdd-error-chains-svelte-5
