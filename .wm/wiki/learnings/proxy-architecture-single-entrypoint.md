---
id: wiki:learnings:proxy-architecture-single-entrypoint
title: 'Learning: MCP Proxy Architecture — Single Entry Point'
type: concept
tags: [learning, architecture, mcp, proxy, entry-point]
status: draft
relates_to:
  - {type: references, target: wiki:learnings:multi-crate-separation}
  - {type: references, target: wiki:specs:web-server-build-serve}
  - {type: references, target: wiki:patterns:mcp-http-proxy}
  - {type: references, target: wiki:patterns:mcp-proxy-singleton}
  - {type: references, target: wiki:tasks:srv-create-mcp-proxy-with-static-tool-list}
---

# Learning: MCP Proxy Architecture — Single Entry Point

## Decisions

### Single Binary Entry Point (GOOD_CALL)
- **Chose:** `wm-cli` as the only standalone binary. `wm-server` and `wm-vectors-bin` are library crates only (no `src/main.rs`, no `[bin]` table in Cargo.toml).
- **Over:** Separate binaries for `wm-server` (HTTP API), `wm-mcp` (MCP proxy), `wm-cli` (CLI)
- **Tag:** GOOD_CALL
- **Outcome:** One binary to build, one binary to ship, no coordination between processes. The `wm-cli mcp` command creates the engine, spawns the HTTP server in-process on a random port, then runs the MCP proxy against it — all in one process. The `wm-cli web` command does the same but serves the Angular UI on a user-specified port.
- **Recommendation:** For CLI tools with multiple protocol adapters (MCP, HTTP, Web UI), keep one entry point binary that embeds everything. Use library crates for the shared logic.

### Generic /api/tools Dispatch (GOOD_CALL)
- **Chose:** A single `POST /api/tools` endpoint that takes `{"name": "<tool>", "arguments": {}}` and dispatches through the ToolRegistry.
- **Over:** 58 individual REST routes, one per MCP tool.
- **Tag:** GOOD_CALL
- **Outcome:** The MCP proxy registers 78 tool handlers dynamically by reading the ToolRegistry's tool list. Each handler posts to the same `/api/tools` endpoint. No need to add new REST routes when new tools are added to `wm_core::mcp::tools`. The web UI still uses specific convenience routes (`/api/search`, `/api/pages/list`) for readability, but all MCP tools work through the generic dispatch.
- **Recommendation:** When building an MCP proxy to an HTTP backend, use a single generic dispatch endpoint rather than one route per tool.

### Global OnceLock&lt;ToolRegistry&gt; (TRADEOFF)
- **Chose:** A global `OnceLock<Arc<ToolRegistry>>` static for the tool registry, separate from the axum `AppState`.
- **Over:** Including `Arc<ToolRegistry>` in `AppState` alongside `Arc<EngineState>`.
- **Tag:** TRADEOFF
- **Outcome:** Avoiding axum's `Router<S>` type constraints. `AppState` with `tools: Arc<ToolRegistry>` made `Router<AppState>` unable to call `.into_make_service()` because the trait bound `S: Clone + Send + Sync + 'static` couldn't be satisfied (likely a transitive bound issue with the closure types in ToolRegistry). Moving tools to a global `OnceLock` worked around this cleanly, since there's only one server process and the ToolRegistry is initialized once at startup.
- **Recommendation:** When axum's `Router<S>` type constraints fight you, consider a global singleton for truly single-instance state. Only do this when the state is initialized once and never changes.

## Failures

### reqwest::blocking::Client Panic in Tokio Context
- **What went wrong:** `reqwest::blocking::Client::new()` panicked with "Cannot drop a runtime in a context where blocking is not allowed" when called inside a `#[tokio::main]` async context.
- **Root cause:** `reqwest::blocking::Client` creates its own tokio runtime internally. Creating or dropping a tokio runtime inside an existing tokio runtime panics.
- **Time lost:** ~30 minutes debugging + refactoring
- **Prevention:** Use `ureq` (pure blocking HTTP, no tokio dependency) instead of `reqwest::blocking` in tokio contexts. If you must use `reqwest::blocking`, create the client in a separate thread via `std::thread::spawn(|| reqwest::blocking::Client::new()).join().unwrap()`.

### axum Router&lt;S&gt; Type Constraint
- **What went wrong:** Adding `tools: Arc<ToolRegistry>` to axum's `AppState` broke `Router::into_make_service()` — the method was "not found" for `Router<AppState>`.
- **Root cause:** `into_make_service()` is defined in `impl<S> Router<S> where S: Clone + Send + Sync + 'static`. Something in `ToolRegistry` (likely the closure types in registrations made by `register_all_tools`) violated these bounds transitively.
- **Time lost:** ~45 minutes across multiple failed attempts
- **Prevention:** Keep axum state simple. Use global statics for complex state like ToolRegistry. Axum's type system fights complex state types — keep AppState minimal.

## Related

- @wiki/learnings/multi-crate-separation — Original crate separation decision
- @wiki/concepts/specs/web-server-build-serve — Web server spec
- @wiki/concepts/patterns/mcp-http-proxy — MCP proxy pattern