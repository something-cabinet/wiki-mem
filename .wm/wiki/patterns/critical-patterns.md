---
title: Critical Patterns
page_type: pattern
id: concepts/critical-patterns
tags:
  - learning
  - critical
---

# Critical Patterns

Promoted learnings from completed work. Read this at the start of every session via wm-init. These are lessons that cost the most to learn and save the most by knowing.

---

## [2026-06-16] ArcSwap Co-Swap for Graph State
**Category:** decision
**Source:** @wiki/tasks/awotvr, @wiki/tasks/r8n30s
**Tags:** [graph, concurrency, arc-swap]

When you have a data structure (like a graph) that's rebuilt entirely on mutation, and you have concurrent readers: DON'T use RwLock. Build the new version in background, then atomically swap via ArcSwap. Readers hold an Arc to the old snapshot and never block. The graph and its id_index must be co-swapped atomically to prevent dangling NodeIndex references.

**Full entry:** @wiki/learnings/learning-wiki-mem-graph-architecture

---

## [2026-06-16] SQLite Bundled When Already Using ONNX — SUPERSEDED
**Category:** decision (overridden)
**Source:** @wiki/tasks/g2gckv
**Tags:** [storage, sqlite, vectors, superseded]

**Superseded 2026-07-04:** Use flat binary `vectors.bin` instead. Vectors are derivable from wiki pages (no data loss on corruption). Adding `rusqlite` bundled costs ~30s compile time and adds another C dependency to audit. The vectors.bin format at 200 lines is simpler, avoids schema migrations, and the mmap approach (memmap2) keeps memory usage low. WAL crash safety doesn't help when the source of truth is elsewhere.

**Original:** If ort (ONNX Runtime) is already in your crate stack, rusqlite bundled adds zero marginal C dependency burden. Use SQLite for vector persistence — WAL mode crash safety, incremental upserts, schema migrations. Don't write a custom flat binary format just to avoid "a database." The 4ms startup overhead vs flat binary is negligible.

**Full entry:** @wiki/learnings/learning-wiki-mem-graph-architecture

---

## [2026-06-16] Prefix MCP Tools to Avoid Collisions
**Category:** decision
**Source:** @wiki/tasks/j4tx6c
**Tags:** [mcp, naming]

Host apps (OpenCode, Kiro) may have built-in tools with generic names like `code`, `search`, `page`. Always prefix your MCP tools with a project-specific namespace (`wm_`, `gh_`, etc.). The prefix is explicit in the tool registration — don't rely on the client to namespace.

**Full entry:** @wiki/learnings/learning-wiki-mem-graph-architecture

---

## [2026-06-16] CLI Is Bootstrap, MCP Is the Product — SUPERSEDED
**Category:** decision (overridden)
**Source:** @wiki/tasks/j4tx6c
**Tags:** [cli, mcp, architecture, superseded]

**Superseded 2026-07-04:** The CLI IS the product — TUI-first with Ratatui. Inspired by Knowns (Bubble Tea), the CLI provides instant, keyboard-driven access to the full wiki: search, browse, graph, tasks, time tracking, lint, source management. Interactive TUI when running in a terminal (auto-detected), plain text or JSON when piped. MCP remains the integration protocol for AI agents (OpenCode, Claude Code, Codex), but the CLI is the primary human interface.

**Original:** Only build enough CLI to bootstrap the system (init, serve, version). Everything else goes through MCP. A full CLI creates maintenance burden and delays MCP adoption. The CLI exists for testing and one-time setup, not as a primary interface.

**Why superseded:** The Knowns/Bubble Tea model proved that a TUI-first CLI is the best developer experience. AI agents use MCP natively, but humans reach for the terminal. A rich CLI with Ratatui TUI gives instant feedback (no browser, no server) while the web UI (SvelteKit/wm-ui) is optional for rich visualization. The CLI and MCP share the same engine — tools.rs handlers are called by both, so there's no duplication or drift.

**Full entry:** @wiki/learnings/learning-wiki-mem-graph-architecture

---

## [2026-07-06] MCP Bridge Pattern for Web UIs
**Category:** pattern
**Source:** @wiki/tasks/umpd47, @wiki/tasks/s2ff4x
**Tags:** [mcp, web-ui, architecture]

When building a web UI on top of an MCP-based Rust backend: use a thin Node.js bridge that spawns the Rust process as a child and pipes JSON-RPC 2.0 over stdin/stdout. Don't add an HTTP server crate to the Rust code. The bridge (`wm-bridge.ts`) is ~50 lines, handles `sendRequest()` → write to stdin → read from stdout. SvelteKit API routes delegate to this bridge. This keeps the Rust backend pure MCP and the protocol testable independently.

**Full entry:** @wiki/learnings/learning-post-build-quality-pass-spec-alignment-tui-mcp-integration

---

## [2026-07-06] Fixing Pre-Release Crate API Drift
**Category:** failure
**Source:** @wiki/tasks/kq0kld
**Tags:** [dependencies, onnx, embed]

When `ort 2.0.0-rc.12` API drifts from your code (13 compile errors): (1) Check the crate's actual `src/` files, not its published docs — docs lag behind rc releases. (2) Try the simplest data format first — `(Vec<i64>, Vec<i64>)` tuples for tensor creation, before fighting `ndarray` + `OwnedTensorArrayData` trait implementations. (3) Build `--features embed` in CI to catch drift early. The final fix used `Tensor::from_array((shape, data))` with plain Vecs — simpler than ndarray and fully compatible.

**Full entry:** @wiki/learnings/learning-post-build-quality-pass-spec-alignment-tui-mcp-integration

---

## [2026-07-07] Knowns Core = Memory Layer, Not Spec System
**Category:** decision
**Source:** @wiki/tasks/29fizw
**Tags:** [memory, knowns, architecture, openspec]

OpenSpec (@fission-ai/openspec, 59k stars) is a dedicated spec system with change folders and lifecycle. Knowns/WM specs are a thin technique (Socratic exploration + doc template). The real value is the **memory substrate**: typed graph edges, semantic retrieval, context assembly, cross-references, AC tracking. WM should double down on the memory layer — not compete with OpenSpec on spec lifecycle management.

**Full entry:** @wiki/learnings/learning-knowns-memory-layer-not-a-spec-system

---

## [2026-07-07] Per-Type BM25 + RRF + FSRS Recency
**Category:** pattern
**Source:** @spec/cross-entity-hybrid-search
**Tags:** [search, ranking, fsrs, rrf]

When searching across heterogeneous entity types (pages + memory), use per-type BM25 indexes merged via RRF (not unified index). Tasks stay in the page index with FSRS-6 recency boost — not a separate task index. Memory gets flat text context in retrieve, not graph BFS. Recency model defaults to FSRS-6 (hardcoded 21 params, only stability configurable).

**Full entry:** @wiki/learnings/learning-cross-entity-search-per-type-bm25-fsrs-recency-debounced-indexscheduler

---

## [2026-07-07] Async Write Channel Race — Use Sync Writes
**Category:** failure
**Source:** @wiki/learnings/learning-e2e-test-infrastructure-sync-write-fix
**Tags:** [write-channel, async, tokio, race]

For single-user local tools using tokio, do NOT route file writes through async channels. The return-before-flush semantic causes races with any operation that reads from disk. Use `std::fs::write()` directly. If you must use async writes, add a flush barrier with proper synchronization that doesn't deadlock the tokio runtime (use `spawn_blocking`, not blocking on worker threads).

**Full entry:** @wiki/learnings/learning-e2e-test-infrastructure-sync-write-fix

---

## [2026-07-07] tools.rs Split Into Domain Modules
**Category:** pattern
**Source:** @spec/architectural-refactors-toolsrs-split-dependency-inversion-extraction
**Tags:** [architecture, refactor, mcp, tools]

When `mcp/tools.rs` exceeds ~1000 lines, split it into per-domain modules under `mcp/tools/`. Each domain module has a `pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>)` function. The parent `tools.rs` becomes a ~30-line delegator calling each module via `pub mod search; pub use search::*;`. Domain names match tool prefixes (`search.rs` → `wm_search.*`, `page.rs` → `wm_page.*`). This kept handler code discoverable, prevented merge conflicts, and reduced cognitive load. The old tools.rs was 1769 lines across 14 domains — now each module is 100-250 lines.

**Full entry:** @wiki/learnings/learning-gehenna-app-cross-project-patterns-cdd-error-chains-svelte-5

---

## [2026-07-07] ToolError Typed Error Chaining
**Category:** pattern
**Source:** @wiki/learnings/learning-gehenna-app-cross-project-patterns-cdd-error-chains-svelte-5
**Tags:** [error-handling, rust, toolerror]

ToolError should carry `source: Option<Box<dyn StdError>>` to preserve the full error context chain. Use specific constructors instead of generic `internal()`: `io_error(op, path, err)` for I/O failures, `serde_error(op, err)` for serialization, `lock_poisoned(resource)` for mutex poison. Removed PartialEq+Eq derives (incompatible with error chaining). The `Display` impl shows the chain; `Error::source()` returns the underlying error. This pattern comes from gehenna-app's `RepoError` which wraps `sea_orm::DbErr` with `#[source]`.

**Full entry:** @wiki/learnings/learning-gehenna-app-cross-project-patterns-cdd-error-chains-svelte-5

---

## [2026-07-13] MCP Server Must Advertise Tools Capability
**Category:** failure
**Source:** @wiki/learnings/session-skills-alignment-mcp-tools
**Tags:** [mcp, rmcp, tools, discovery]

If your rmcp MCP server's tools don't appear as callable functions in the AI client, check `get_info()` in the `ServerHandler` impl. `ServerCapabilities::default()` sets `tools: None`, which tells the MCP client "I don't support tools" — so the client never calls `tools/list` and 74 registered tools remain invisible. Fix: set `capabilities = ServerCapabilities::builder().enable_tools().build()` in the `ServerInfo` returned by `get_info()`. This is NOT optional — it's the handshake that enables tool discovery.

**Full entry:** @wiki/learnings/session-skills-alignment-mcp-tools

---

## [2026-07-13] MCP Server Should Be a Thin HTTP Proxy, Not an Engine Owner — UPDATED
**Category:** pattern
**Source:** @wiki/concepts/patterns/mcp-http-proxy, @wiki/learnings/proxy-architecture-single-entrypoint
**Tags:** [mcp, architecture, proxy, design, single-binary]

Don't embed your engine (graph, BM25, ONNX embedder) in an MCP server. Build the MCP server as a stateless HTTP proxy — each tool handler is a 3-line `ureq` call to a backend HTTP API. The HTTP server owns the engine; the MCP server just translates protocols. This eliminates duplicate state (~500MB memory), removes startup latency, and gives all clients (Angular, curl, MCP) a single source of truth.

**Updated 2026-07-13:** The MCP proxy and HTTP server live in the same binary (`wm-cli`). The MCP binary (`wm-mcp`) was deleted — `wm-cli` is the single entry point. The HTTP server is embedded in-process on a random port. No separate processes to manage.

**Full entry:** @wiki/concepts/patterns/mcp-http-proxy

---

## [2026-07-13] reqwest::blocking Panics Inside tokio Runtime
**Category:** failure
**Source:** @wiki/learnings/proxy-architecture-single-entrypoint
**Tags:** [tokio, reqwest, ureq, blocking]

`reqwest::blocking::Client::new()` panics with "Cannot drop a runtime in a context where blocking is not allowed" when called inside a `#[tokio::main]` context. The blocking client creates its own tokio runtime internally. Instead, use `ureq` — a pure blocking HTTP client with zero tokio dependency. If you must use `reqwest::blocking`, create it outside the async context via `std::thread::spawn(|| reqwest::blocking::Client::new()).join().unwrap()`.

**Full entry:** @wiki/learnings/proxy-architecture-single-entrypoint

**Full entry:** @wiki/learnings/multi-crate-separation

---

## [2026-07-13] ureq over reqwest::blocking in tokio contexts
**Category:** failure
**Source:** @wiki/learnings/model-rework-cdd-status-enum-page
**Tags:** [ureq, reqwest, tokio, blocking, async]

reqwest::blocking::Client::new() panics when called inside a #[tokio::main] context. It tries to create its own tokio runtime while already inside one. Fix: use ureq (pure blocking, no tokio dep) or spawn creation via std::thread::spawn.

**Full entry:** @wiki/learnings/model-rework-cdd-status-enum-page

---

## [2026-07-13] CDD — enum Page dispatch over Option wrappers
**Category:** pattern
**Source:** @wiki/concepts/specs/status-model-rework
**Tags:** [cdd, rust, types, enum-dispatch]

Prefer enum dispatch over Option wrappers for per-type data in a unified model. enum Page { Task { meta, data }, Concept { meta }, ... } prevents invalid states at compile time — a Concept page physically cannot have TaskData. The graph stores the loose format internally; the public API stays strict.

**Full entry:** @wiki/learnings/model-rework-cdd-status-enum-page

---

## [2026-07-15] Sync↔Async Bridge: block_in_place + Handle::current().block_on()
**Category:** pattern
**Source:** @wiki/learnings/session-model-rework-learnings
**Tags:** [tokio, async, sync, bridge]

When an async-native crate (turso, reqwest) must be called from sync code inside a multi-thread tokio runtime (like `#[tokio::main]`), use `tokio::task::block_in_place(|| Handle::current().block_on(async { ... }))`. This is the official tokio-recommended pattern. For contexts without a runtime (plain `#[test]`), fall back to creating a standalone `Runtime::new()`. Do NOT create a `Runtime::new()` inside an existing runtime — it panics.

**Full entry:** @wiki/learnings/session-model-rework-learnings

---

## [2026-07-15] Research Official Crate Docs Before Building Workarounds
**Category:** failure
**Source:** @wiki/learnings/session-model-rework-learnings
**Tags:** [research, dependencies, tokio, workaround]

Built a 150-line background thread + mpsc channel workaround for turso's tokio runtime conflict before checking turso's official documentation. The `block_in_place` + `Handle::current().block_on()` pattern was documented on docs.rs and used by every real-world project. ~20 minutes lost. Always check the crate's docs.rs and GitHub README before engineering custom runtime bridges.

**Full entry:** @wiki/learnings/session-model-rework-learnings

---

## [2026-07-16] Repository Trait for Filesystem I/O (PageRepo)
**Category:** pattern
**Source:** @wiki/patterns/pagerepo-trait
**Tags:** [testing, filesystem, rust, repository]

When unit-testing code that reads/writes files (YAML frontmatter, config, page CRUD), extract filesystem ops behind a `PageRepo` trait with `FsPageRepo` (production) and `InMemoryPageRepo` (tests). This lets you unit-test complex mutation logic (like `update_page`'s 200+ lines of YAML manipulation) without a real filesystem or a full `EngineState` bootstrap. The public API stays backward-compatible via internal delegation. This is a special case of the Repository pattern — storage-agnostic, not database-specific.

**Full entry:** @wiki/patterns/pagerepo-trait

---

## [2026-07-16] Repository Pattern is Storage-Agnostic
**Category:** decision (correction)
**Source:** @wiki/patterns/learning-gehenna-app-cross-project-patterns-cdd-error-chains-svelte-5
**Tags:** [architecture, repository, service, rust]

Service and Repository are storage-agnostic patterns — they apply to filesystems and in-memory stores just as well as databases. The codebase already has informal repositories (VersionStore, VectorStore). The real question is ROI: introduce 2-3 key traits where testability is the bottleneck (PageRepo, VectorRepo), decompose EngineState into component bundles, but skip full hexagonal architecture. Don't justify architectural decisions with "we don't have a database" — that's a category error.

**Full entry:** @wiki/patterns/learning-gehenna-app-cross-project-patterns-cdd-error-chains-svelte-5

---

## [2026-07-17] Binary Self-Deployment — wm upgrade
**Category:** pattern
**Source:** @wiki/specs/wm-self-install, @wiki/decisions/wm-self-upgrade
**Tags:** [deployment, setup, knowns, path]

When building a CLI tool that generates platform configs (MCP, IDE settings), copy the running binary to `~\.toolname\bin\` and register it on PATH. Platform configs then reference `toolname` by name instead of fragile `target/debug/` paths. Use `REG ADD HKCU\Environment` on Windows, `~/.profile` on Unix. A single `wm init --full` chains upgrade → PATH → config → project init. Pattern matches Knowns' `~\.knowns\bin\knowns.exe` deployment.

**Full entry:** @wiki/patterns/wm-init-full

---

## [2026-07-16] Domain Splitting — "What" Comments Signal Missing Modules
**Category:** pattern
**Source:** @wiki/specs/domain-splits-page-codeintel-template-graph
**Tags:** [architecture, refactor, module-structure, rust]

When a file has section markers (`// ─── Name ───`) that partition distinct concerns, those sections should be files. The section marker IS the module boundary — it signals that the code inside it has a single responsibility that can be named. Trust the signal: if you'd write a "what" comment before a block, that block should be a named function. If you'd write a section marker, that section should be a sub-module file. Applied to 7 files in wm-core (code_intel, template_engine, graph, page, task, template, page tool), totaling ~4000 lines → 30+ files.

**Full entry:** @wiki/specs/domain-splits-page-codeintel-template-graph

---

## [2026-07-17] Crate Extraction with Backward Compat — `pub use wm_foo as foo`
**Category:** pattern
**Source:** @wiki/specs/extract-packages-from-wm-core
**Tags:** [refactor, packages, workspace, architecture, dependency]

When extracting modules from a monolithic crate into standalone packages, maintain backward compatibility by replacing `pub mod foo;` with `pub use wm_foo as foo;` in the original crate's `lib.rs`. This makes all existing `wm_core::foo::Bar` imports resolve transparently to the new package. Key lessons: (1) rename `mod.rs` → `lib.rs` on extraction (packages require lib.rs), (2) move optional deps and feature flags WITH the module — don't leave them in the old crate, (3) keep `[workspace.dependencies]` in sync to avoid manifest parsing failures, (4) test all feature combinations after extraction. Applied to 12 packages extracted from wm-core (~13K → ~4K lines).

**Full entry:** @wiki/patterns/crate-extraction-with-backward-compat

---

## [2026-07-16] MCP Tool Decomposition — action.rs + output.rs + mod.rs
**Category:** pattern
**Source:** @wiki/specs/domain-splits-page-codeintel-template-graph
**Tags:** [mcp, tools, architecture, module-structure]

Every MCP tool file follows the same three-part structure: action enum (serde tagged), output structs, and handler dispatch. Split each tool file into `action.rs` (enum), `output.rs` (structs), and `mod.rs` (handler). The action enum and output structs are data-only; the handler is the only file with behavioral dependencies. This keeps type definitions clean and prevents merge conflicts when multiple agents work on different parts of the same tool. Applied to 3 tool files (task, template, page) — all 722+, 715+, 525+ lines respectively.

**Full entry:** @wiki/specs/domain-splits-page-codeintel-template-graph
