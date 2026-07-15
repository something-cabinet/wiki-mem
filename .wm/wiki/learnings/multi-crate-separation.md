---
title: Learning: Multi-Crate Architecture Separation
page_type: learning
status: draft
tags:
  - learning
  - architecture
  - workspace
  - restructure
---

# Learning: Multi-Crate Architecture Separation

## Patterns

### Separating Protocol Adapters from Engine
- **What:** Split the project into distinct crates by responsibility: `wm-core` (library), `wm-server` (HTTP API library), `wm-cli` (single binary — CLI + MCP proxy + embeds wm-server), `wm-vectors-bin` (vector binary format), `wm-web` (Angular UI). Each crate has one job and one dependency direction. `wm-cli` is the only standalone binary — `wm-server` and `wm-vectors-bin` are library crates only.
- **When to use:** When a monolithic crate has grown to do too many things (library code + MCP server + HTTP server), preventing independent reuse and causing duplicated initialization.
- **Source:** Restructure session — all code moved to `apps/` directory

### Crate Dependency Flow

```
wm-core  ←  wm-server  (lib only, no binary)
wm-core  ←  wm-cli (single binary, embeds wm-server as library)
wm-core  ←  wm-vectors-bin (standalone format crate, zero deps)
wm-cli   →  wm-server (used as library, spawned in-process)
wm-web   →  wm-server (HTTP calls to embedded server)
```

Dependencies flow in one direction: library ← server ← binary. The MCP proxy lives inside `wm-cli` — no separate `wm-mcp` binary needed. All tool calls forward from the MCP stdio transport to the in-process HTTP server via `ureq`.

## Decisions

### MCP Server as Thin HTTP Proxy (GOOD_CALL) — UPDATED
- **Chose:** Pure protocol adapter with zero state, forwarding every tool call to the embedded `wm-server` via `ureq` (pure blocking HTTP, no tokio conflict).
- **Over:** Embedding the full engine in the MCP process OR creating a separate `wm-mcp` binary.
- **Tag:** GOOD_CALL
- **Outcome:** Eliminated duplicate engine initialization (~500MB memory for two ONNX models). The MCP proxy lives in `wm-cli` — the single entry point. No separate binary to manage. The HTTP server is the single source of truth, accessible by any client (MCP, Angular, curl).
- **Recommendation:** Keep MCP stateless and co-located in the same binary as the server. One entry point (`wm-cli`), one engine, one process.

### HTTP Server Separated from Angular UI (GOOD_CALL)
- **Chose:** `wm-web` became a pure Angular project (no Rust backend), `wm-server` owns the HTTP API
- **Over:** Keeping the Axum HTTP routes embedded in the Angular crate's Rust backend
- **Tag:** GOOD_CALL
- **Outcome:** Angular can be developed independently (`ng serve`), the HTTP API is testable via curl/Postman, and the server can be deployed as a standalone daemon without the Angular build. No more `rust-embed` rebuilds for frontend changes.
- **Recommendation:** Never mix frontend bundling and backend API in the same crate. Angular devs shouldn't need Rust tooling.

### apps/ Directory Layout (TRADEOFF)
- **Chose:** `apps/wm-core`, `apps/wm-cli`, `apps/wm-server`, `apps/wm-vectors-bin`, `apps/wm-web`
- **Over:** Flat layout (old: `wm-core/`, `wm-cli/`, `wm-web/`)
- **Tag:** TRADEOFF
- **Outcome:** Cleaner for projects with many sub-crates. Adds one directory level but makes it obvious where application code lives versus config/docs.
- **Recommendation:** Use `apps/` for workspaces with 3+ crates. Update `Cargo.toml` workspace members and all internal `path =` deps.

## Failures

### MCP Tool Discovery Failure
- **What:** 74 MCP tools were invisible to the AI client
- **Root cause:** Missing `ServerCapabilities::builder().enable_tools()` in the initialize handshake
- **Time lost:** ~30 minutes debugging before finding the missing capability flag
- **Prevention:** Always audit MCP server `get_info()` for `ServerCapabilities` when tools don't appear. The default `capabilities.tools: None` tells the client "no tools available."

### Missing required field: ToolError partial move
- **What:** `From<ToolError> for ErrorData` impl had a partial move error — `err.message` was moved before `err.to_json()` could borrow it
- **Root cause:** The compiler caught a use-after-move that would cause incorrect error messages
- **Time lost:** 2 minutes to fix (extract `let json = err.to_json()` first)
- **Prevention:** When converting error types where `message` is moved but other methods need the full struct, extract the computed values first.
### Pre-existing API drift in wm-cli and wm-web

- **What:** `wm-cli` and `wm-web` called functions that had been renamed/refactored in `wm-core`: `search::query` → `search::query::run_unified_search`, `task_board` → `build_task_board`, `rebuild_snapshot` → `rebuild_graph_snapshot`, etc.
- **Root cause:** The crates weren't being compiled regularly (`cargo build` was not run on the full workspace), allowing drift to accumulate silently
- **Time lost:** ~20 minutes across 15 fixes
- **Prevention:** Run `cargo build --workspace --no-default-features` in CI and before merging. Always rebuild the full workspace when refactoring core library APIs.
