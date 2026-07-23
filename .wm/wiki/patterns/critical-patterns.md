---
title: Critical Patterns
type: pattern
tags: [learning, critical]
relates_to:
  - {type: references, target: wiki:decisions:model-methods-over-scattered-mappings}
---

---
title: Critical Patterns
type: pattern
tags:
- learning
- critical
relates_to:
  - {type: references, target: wiki:decisions:model-methods-over-scattered-mappings}
---

---
title: Critical Patterns
page_type: pattern
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

## [2026-07-06] Post-RRF Rerank — Put Boosts After Fusion, Not Before
**Category:** pattern
**Source:** @doc/specs/local-knowledge-engine
**Tags:** [search, ranking, rrf, rerank]

BM25 rerank boosts applied before RRF fusion are silently discarded — RRF only uses ranks, not raw scores. Move all rerank heuristics (title density, exact match, tag overlap) to a post-RRF stage. Use Knowns-inspired additive bonuses: title density +0.03/word, exact title +0.15, proportional tag overlap, exact ID +0.10.

**Full entry:** @wiki/patterns/post-rrf-rerank

---

## [2026-07-21] Spartan UI Select: Always hlmSelect + *hlmSelectPortal
**Category:** failure
**Source:** @doc/concepts/hlmselect-portal-ng-container
**Tags:** [angular, spartan-ui, select]

Every select in the app was broken by three violations: (1) `<div brnSelect>` without `hlmSelect` — no popover overlay. (2) `<ng-container hlmSelectPortal>` — `BrnPopoverContent` needs a TemplateRef, `<ng-container>` doesn't provide one. (3) Missing `*` prefix on structural directive. Always use `<div hlmSelect>` + `<hlm-select-content *hlmSelectPortal>`.

**Full entry:** @wiki/concepts/hlmselect-portal-ng-container

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

## [2026-07-13] MCP Server Must Advertise Tools Capability
**Category:** failure
**Source:** @wiki/learnings/session-skills-alignment-mcp-tools
**Tags:** [mcp, rmcp, tools, discovery]

If your rmcp MCP server's tools don't appear as callable functions in the AI client, check `get_info()` in the `ServerHandler` impl. `ServerCapabilities::default()` sets `tools: None`, which tells the MCP client "I don't support tools" — so the client never calls `tools/list` and 74 registered tools remain invisible. Fix: set `capabilities = ServerCapabilities::builder().enable_tools().build()`.

**Full entry:** @wiki/learnings/session-skills-alignment-mcp-tools

---

## [2026-07-16] Repository Pattern is Storage-Agnostic
**Category:** decision (correction)
**Source:** @wiki/patterns/learning-gehenna-app-cross-project-patterns-cdd-error-chains-svelte-5
**Tags:** [architecture, repository, service, rust]

Service and Repository are storage-agnostic patterns — they apply to filesystems and in-memory stores just as well as databases.

**Full entry:** @wiki/patterns/learning-gehenna-app-cross-project-patterns-cdd-error-chains-svelte-5

---

## [2026-07-18] Workspace Dependency Unification
**Category:** pattern
**Source:** ad-hoc session (target bloat diagnosis)
**Tags:** [cargo, workspace, build]

Every crate in a Rust workspace MUST use `{ workspace = true }` for shared dependencies. Inline versions cause Cargo to compile the same dependency multiple times. A 16-crate workspace had 304 .lib files (18.6 GB) — after fixing 29 inline deps, clean build is 1.62 GB.

**Full entry:** @wiki/patterns/workspace-dep-unification

---

## [2026-07-20] MCP Proxy Singleton — Share One EngineState
**Category:** pattern
**Source:** @doc/specs/single-http-server
**Tags:** [architecture, mcp, proxy, singleton, server]

When multiple processes need the same engine (MCP, CLI, Web UI): don't create separate EngineState copies. Use a singleton daemon pattern — health-check the server before starting, spawn if down, connect if alive. One `GET /api/health` to decide. Saves ~500MB per process and eliminates stale-data bugs.

**Full entry:** @wiki/patterns/mcp-proxy-singleton

---

## [2026-07-20] MCP Tool Unavailability — Stop Retrying, File Directly
**Category:** failure
**Source:** @doc/rules/tool-reliability-bug-tracking
**Tags:** [mcp, fallback, tooling, failure]

When MCP tools fail: retry at most twice, then switch to filesystem fallback and create a bug task. Don't keep retrying.

**Full entry:** @wiki/concepts/mcp-tool-unavailability-fallback

---

## [2026-07-22] EnginePort — Abstract Backend Transport in Angular
**Category:** pattern
**Source:** @wiki/specs/http-wasm-architecture-cleanup
**Tags:** [angular, architecture, testing, di]

Define an `EnginePort` as an Angular `InjectionToken<EnginePort>` interface. Consumers depend on the interface, not the implementation. Production uses `HttpEngineService` (fetch); tests use `MockEngineService` (canned typed responses). This gives you component-testability, transport-swappability, and typed responses (zero `any`) without changing the actual transport. Takes ~1 day to introduce and saves hours of testing frustration.

**Full entry:** @wiki/patterns/engine-port-backend-abstraction

---

## [2026-07-22] WASM Crate Integration (fjadra profile)
**Category:** pattern
**Source:** @wiki/specs/http-wasm-architecture-cleanup
**Tags:** [wasm, angular, build, integration]

When adding WASM to an Angular app, the fjadra profile is the gold standard: the crate must be cdylib + wasm-bindgen + serde, fs-free, tokio-free, rayon-optional, and pure computation. Built with `wasm-pack build --target web`, output in Angular assets, lazy-loaded via dynamic `import()`. Never wrap the entire engine — only pure, stateless, CPU-bound candidates that would be chatty over HTTP. The fjadra-wasm → graph-algo-wasm → bm25-rerank-wasm → md-parse-wasm sequence proved this pattern is repeatable.

**Full entry:** @wiki/patterns/wasm-crate-integration

---

## [2026-07-22] Keep HTTP; WASM for Pure Compute Only
**Category:** decision
**Source:** @wiki/specs/http-wasm-architecture-cleanup
**Tags:** [architecture, wasm, http, angular, rust]

HTTP is the correct seam between a stateful Rust engine (filesystem, threads, SQLite, ONNX, subprocesses) and a stateless browser client. WASM cannot replace it — wm-core doesn't compile to wasm32-unknown-unknown (tokio::fs, ray on, ort, turso, walkdir, subprocess), the data model is files on disk (OPFS would strand it from agents), and a browser WASM EngineState would diverge from the daemon's. Use WASM only for pure-compute extensions that would be chatty over HTTP. Each WASM addition must delete its HTTP predecessor.

**Full entry:** @wiki/decisions/http-wasm-seam

---

## [2026-07-23] wm_help Must Read Tool Schemas From ToolRegistry
**Category:** decision
**Source:** @wiki/tasks/embed-shim-templates
**Tags:** [mcp, tools, schemas, maintenance]

Don't maintain a hardcoded tool list for wm_help. The ToolRegistry already stores descriptions and JSON schemas for every registered tool, generated automatically via schemars derives. Have wm_help read dynamically from the registry via EngineState.tool_list. This keeps parameter schemas always in sync, eliminates stale docs, and lets agents discover required fields. The old hardcoded 50-entry list was always out of date and didn't include schemas.

**Full entry:** @wiki/decisions/wm-help-tool-registry

---

## [2026-07-23] Register MCP Handlers Directly, Never Proxy Through HTTP
**Category:** decision
**Source:** @wiki/specs/mcp-direct-handlers
**Tags:** [mcp, architecture, proxy, tool-registry]

Don't maintain a separate proxy layer or hardcoded tool list for MCP. Create the engine in-process, call `register_all_tools()` on the registry, and serve stdio directly. A proxy duplicates registration (guaranteed drift), hides real tools from clients, serves empty schemas, adds latency, and creates a runtime dependency on a separate HTTP server. The old proxy's STATIC_TOOLS had ~26 dead names and ~25 invisible tools — it was silently broken. Also: tool errors must use `isError: true` (not JSON-RPC protocol errors) per the MCP spec, which direct handlers enable naturally.

**Full entry:** @wiki/decisions/mcp-direct-handlers-over-proxy

---

## [2026-07-23] Identical-Function → Generic Composition
**Category:** pattern
**Source:** @wiki/tasks:task-uc9ioi-architectural-refactors-toolsrs-split-skill-dependency-method-extraction
**Tags:** [refactoring, boilerplate, composition, rust]

When you spot 3+ functions with identical structure (same control flow, same error handling, same result building) that only differ by data, extract a private generic function parameterized over the varying data. Each variant becomes a thin wrapper that only defines its data. This eliminated ~120 lines of copy-paste from symbols_helper.rs (7 for_* functions → 1 generic + 7 wrappers). Works across any language — the signal is structural identity with only data variation.

**Full entry:** @wiki/patterns/identical-function-composition

---

## [2026-07-23] Model Methods > Scattered Mapping Functions
**Category:** decision
**Source:** @wiki/tasks:edge-type-pruning
**Tags:** [architecture, rust, serde, enum]

When an enum's string representation (serde, YAML, display) is mapped in 3+ separate functions across different modules, the representations will drift. Move `to_str()` and `from_str()` methods onto the model itself. This eliminates import overhead, makes the mapping discoverable via `TypeName::`, and provides a single source of truth. Applied to EdgeType — removed 3 functions, unified alias handling, and eliminated variant drift.

**Full entry:** @wiki/decisions/model-methods-over-scattered-mappings

---

## [2026-07-23] CLI Must Run Directly, Never Proxy Through HTTP
**Category:** decision
**Source:** @wiki/tasks/refactor-wm-cli-mcp-to-register-handlers-directly
**Tags:** [cli, architecture, proxy, http]

The CLI was refactored to proxy all page/graph/task operations through HTTP to a wm-server daemon. This broke offline operation, all integration tests (35→14 pass), and added latency. Fix: CLI must never proxy through HTTP — use `create_engine()` + direct `wm_core::*` API calls in-process. The HTTP daemon is for web UI and remote access only; the CLI is for local direct use. Removing the proxy also removed the `ureq` dependency.

**Full entry:** @wiki/decisions/cli-direct-execution-not-http-proxy

---

## [2026-07-23] Separate Service Ports over Monolithic EnginePort
**Category:** decision
**Source:** deepwork session (UI review + code intel search)
**Tags:** [angular, architecture, services, engineport]

Each distinct API domain gets its own port interface + InjectionToken + HTTP impl + mock impl, rather than adding every method to a monolithic `EnginePort`. Avoids interface bloat, mock contamination, and allows independent evolution. Applied in `CodeIntelPort`. New Angular domains should follow this pattern rather than extending `EnginePort`.

**Full entry:** @wiki/decisions/separate-service-ports-over-monolithic-engineport

---

## [2026-07-23] HTTP Services Must Unwrap {success, data} Envelope
**Category:** failure
**Source:** deepwork session (code intel search review)
**Tags:** [angular, http, api, consistency]

Two HTTP service implementations handled the server response envelope differently — one returned raw JSON, the other unwrapped `{success, data}`. The inconsistent approach caused silent `undefined` data reads that fell through to `|| ''` fallbacks and were invisible until Oracle review. Convention: every `httpCall` must extract `{success, data}`, throw on `!success`, and return `data` typed as `T`. Extract a shared helper rather than duplicating across services.

**Full entry:** @wiki/concepts/response-envelope-inconsistency