---
id: wiki:learnings:multi-crate-separation
type: concept
title: 'Learning: Multi-Crate Architecture Separation'
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

### MCP Server as Thin HTTP Proxy (SUPERSEDED 2026-08-12)
- **Chose (now superseded):** a pure protocol adapter forwarding every tool call to a `wm-server` daemon over HTTP.
- **Superseded by:** in-process dispatch. `wm-cli mcp` now hosts `ToolRegistry` directly over rmcp stdio - no daemon, no HTTP proxy, no tokens. The proxy was a compensating layer (the CLI became an HTTP client to its own functions); ~5,300 lines were deleted. See @wiki/rules/no-compensating-layers.
- **Lesson that survived:** one entry point, one engine, one process - still true, now achieved by in-process dispatch rather than a proxy.

### HTTP Server Separated from Angular UI (SUPERSEDED 2026-10-15)
- **Chose (now superseded):** `wm-web` as a pure Angular project with `wm-server` owning the HTTP API.
- **Superseded by:** retirement of `wm-server` and `wm-web` - WM is now a single-binary, single-root, in-process tool with no bundled web UI and no HTTP surface.
- **Lesson that survived:** never mix frontend bundling with a backend API in one crate - sound guidance for any future web client, though no such server exists today.

### apps/ Directory Layout (TRADEOFF)
- **Chose:** `apps/wm-core`, `apps/wm-cli`, plus `packages/*` shared libraries. (`apps/wm-server`, `apps/wm-vectors-bin`, `apps/wm-web` were retired 2026-10-15; `packages/wm-tools` was added 2026-10-16.)
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

### Moving tests between crates silently breaks CI target references

- **What:** Extracting `mcp/**` moved the integration suites (`mcp_test`, `security_test`, `e2e_mcp`, `semantic_test`, `error_data_transport_test`) from `apps/wm-core` to `packages/wm-tools`, but `.github/workflows/ci.yml` still invoked them with `-p wm-core --test <name>`. Those jobs stopped resolving their targets and would have failed on the next CI run.
- **Root cause:** A file move between crates changes the *build graph*, not just the filesystem; the CI command lines encode `-p <crate>`, and nobody updated them.
- **Time lost:** ~15 minutes, found during the flow's own verification rather than by CI.
- **Prevention:** When a test file moves crates, grep CI for its name and repoint the `-p` target in the same change. A moved test that no longer runs is a silent coverage hole.
