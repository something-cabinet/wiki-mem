---
title: Single HTTP Server — Replace Tauri with wm-server Daemon
type: spec
---

---
title: Single HTTP Server — Replace Tauri with wm-server Daemon
type: spec
tags: [spec, architecture, server, http, tauri-removal]
---

## Overview

Currently, every WM process creates its own `EngineState`:
- `wm-cli mcp` — in-process MCP handlers + full engine
- `wm-cli <cmd>` — per-command engine instance
- Tauri backend — separate engine instance (15 hand-written commands)
- **Total: 3+ copies of the engine**, each with its own graph, BM25 index, embedder, and memory

This spec replaces all of that with a single `wm-server` daemon that owns the engine and serves both the HTTP API and the Angular SPA. MCP becomes a thin stdio-to-HTTP proxy. Tauri is removed entirely.

## Locked Decisions

- D1: A single `wm-server` binary owns the **one** `EngineState`. API on `127.0.0.1:4090` (plain HTTP).
- D2: `wm-cli mcp` becomes a **thin HTTP proxy** — static tool list at compile time, lazy HTTP dispatch. No in-process EngineState.
- D3: Angular frontend becomes a **pure web app** — replaces `window.__TAURI_INTERNALS__.invoke` with standard `fetch()` to `:4090`.
- D4: **Tauri removed** — `apps/wm-web/src-tauri/` deleted. `wm-server` serves the embedded Angular SPA with `rust-embed`.
- D5: **Singleton daemon** — `.wm/server.json` written on startup. `wm-cli mcp` checks health, spawns if down, connects if alive.
- D6: **fjadra dep** moves from `src-tauri/Cargo.toml` to `wm-server/Cargo.toml`. `graph_rebuild_loop` moves to wm-server during Phase 1.
- D7: **Graph layout SSE** is job-scoped — `POST /api/graph/layout → {job_id}`, `GET /api/graph/layout/{job_id}/events` streams positions.
- D8: **MCP transport** moves from `wm-core::mcp` to `wm-cli` before Phase 3 (enables Tauri deletion).
- D9: **Migration order**: (1) move transport to cli, (2) wm-server with routes, (3) wire Angular to HTTP, (4) MCP proxy, (5) delete Tauri, (6) CLI migration.
- D10: **3 contradictory docs** fixed before code: enterprise-grade.md D1, web-server-build-serve.md (superseded), axum-over-rocket.md (annotate superseded line).

## Requirements

### Functional Requirements

- FR-1: `wm-server` starts, binds to `127.0.0.1:4090`, responds to `GET /api/health` with `200 OK`
- FR-2: `wm-server` serves the Angular SPA at `GET /` (embedded via `rust-embed`)
- FR-3: All MCP tools are exposed as RESTful HTTP endpoints under `/api/`
- FR-4: Job-scoped SSE for graph layout at `GET /api/graph/layout/{job_id}/events`
- FR-5: Global SSE event stream at `GET /api/events` for real-time page/memory/task updates
- FR-6: `wm-cli mcp` starts rmcp on stdio, registers static proxy handler list (compile-time), spawns `wm-server` as child process if not running
- FR-7: Angular `ApiService` calls `fetch('http://localhost:4090/api/...')` instead of Tauri `invoke()`
- FR-8: `wm-cli <cmd>` operations use HTTP calls to `:4090` where possible (local-only: `wm init`, `wm setup`)
- FR-9: `wm-server.exe` opens the default browser on startup (`open::that("http://localhost:4090")`)
- FR-10: `graph_rebuild_loop` runs in wm-server background task (moved from Tauri setup())
- FR-11: `.wm/server.json` discovery: server writes port+pid, clients read and health-check

### Non-Functional Requirements

- NFR-1: Localhost HTTP latency <1ms — no perceptible difference from in-process calls
- NFR-2: Build must pass with zero errors
- NFR-3: All existing E2E journeys must pass (adapted for HTTP client)
- NFR-4: Angular SPA embeddable in `wm-server` binary (same `rust-embed` pattern as skills)
- NFR-5: Memory footprint drops from ~3× EngineState to 1×
- NFR-6: MCP handshake does NOT block on server spawn (static tools + parallel spawn)

## Acceptance Criteria

- [ ] AC-1: `cargo run -p wm-server` starts on `:4090`, `GET /api/health` returns `200`
- [ ] AC-2: `GET http://localhost:4090/` serves the Angular app — all views render
- [ ] AC-3: `POST /api/search/query { q: "test" }` returns same results as current `wm_search.query` MCP tool
- [ ] AC-4: `wm-cli mcp` starts with server down → spawns it → registers static tools → handles MCP calls
- [ ] AC-5: Angular search, task board, pages, memory, graph views work via `fetch()` without Tauri
- [ ] AC-6: Graph layout SSE streams job-scoped positions (coarse → refine → settled)
- [ ] AC-7: `graph_rebuild_loop` runs in wm-server, keeps the graph current
- [ ] AC-8: Running `wm-server` a second time detects existing process and exits cleanly
- [ ] AC-9: Tauri crate (`apps/wm-web/src-tauri/`) deleted without breaking build
- [ ] AC-10: `wm-core::mcp` module removed (transport moved to wm-cli)
- [ ] AC-11: All existing E2E journeys pass with new HTTP client
- [ ] AC-12: 3 contradictory docs fixed (enterprise-grade D1, web-server-build-serve, axum-over-rocket)

## Scenarios

### Scenario 1: User starts the app
**Given** the user runs `cargo run -p wm-server`
**Then** the server starts on `127.0.0.1:4090`
**And** writes `.wm/server.json`
**And** the default browser opens to `http://localhost:4090`
**And** the Angular app loads with all views functional

### Scenario 2: AI agent connects via MCP
**Given** an AI agent launches `wm-cli mcp`
**When** MCP checks `GET /api/health`
**If** :4090 responds → connects as proxy (static tools)
**If** :4090 is down → spawns `wm-server` as child, connects after health check
**Then** the agent uses all tools via MCP, each proxied to the same server

### Scenario 3: Concurrent access
**Given** the user is browsing the Angular UI
**When** an AI agent makes a mutation via MCP
**Then** the change is reflected in the same EngineState
**And** the Angular UI receives an SSE event
**And** the new data appears without manual refresh

### Scenario 4: Graph layout
**Given** the graph view is open
**When** the user opens the graph
**Then** `POST /api/graph/layout` starts a fjadra job
**And** `GET /api/graph/layout/{job_id}/events` streams progressive positions via SSE
**And** the canvas updates in real-time

## Migration Phases

### Phase 0: Fix docs + move transport
1. Fix 3 contradictory architecture docs
2. Move `serve_rmcp`, `ToolRegistry` from `wm-core::mcp` to `wm-cli`

### Phase 1: Create wm-server (alongside existing stack)
1. `cargo init apps/wm-server`
2. Add axum, tower-http, wm-engine deps
3. Implement router with full route surface
4. Embed Angular SPA via `rust-embed`
5. Move `graph_rebuild_loop` from Tauri to wm-server background task
6. Move `fjadra` dep from `src-tauri/Cargo.toml` to `wm-server/Cargo.toml`
7. Verify: server starts, API works, Angular renders in browser

### Phase 2: Wire Angular to HTTP
1. Update `proxy.conf.json` for dev (`/api` → `:4090`)
2. Rewrite `api.service.ts` to use `fetch()` (replace `tauriInvoke`)
3. Rewrite `graph-view.component.ts startLayout()` for job-scoped SSE
4. Remove Tauri `invoke()` and `@tauri-apps/api` dependency
5. Verify: `ng serve` + `wm-server` together, all views functional

### Phase 3: Thin MCP proxy
1. Add `ureq` (not reqwest::blocking — known tokio conflict) to `wm-cli`
2. Create `mcp_proxy.rs` with static tool list (compile-time), lazy HTTP dispatch
3. Spawn `wm-server` as child process on health-check failure, parallel with rmcp handshake
4. Tool business logic removed from wm-cli — routes live in wm-server

### Phase 4: Remove Tauri
1. Delete `apps/wm-web/src-tauri/`
2. Remove tauri deps from workspace Cargo.toml
3. Remove `apps/wm-web` from workspace members (keep as standalone npm project)
4. Verify: `cargo build` passes — zero Tauri references

### Phase 5: CLI migration
1. Migrate `wm-cli` commands to use HTTP where applicable
2. Keep local-only: `wm init`, `wm setup`, `wm upgrade`
3. Ratatui TUI connects to HTTP engine

## Technical Notes

### Job-scoped graph layout SSE

```typescript
// Angular: graph-view.component.ts
const jobId = await fetch('POST /api/graph/layout', { nodes, edges }).then(r => r.json());
const source = new EventSource(`/api/graph/layout/${jobId}/events`);
source.addEventListener('graph-coarse', (e) => applyPositions(JSON.parse(e.data)));
source.addEventListener('graph-refine', (e) => applyPositions(JSON.parse(e.data)));
source.addEventListener('graph-settled', (e) => { applyPositions(JSON.parse(e.data)); source.close(); });
```

### Angular HTTP client

```typescript
// api.service.ts — replace tauriInvoke with httpCall
private async httpCall<T>(action: string, body?: unknown): Promise<T> {
  const res = await fetch(`http://localhost:4090/api/${action}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: body ? JSON.stringify(body) : undefined,
  });
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}
```

### MCP proxy (static tool list)

```rust
// wm-cli/src/mcp_proxy.rs
const STATIC_TOOLS: &[&str] = &[
    "wm_initial", "wm_search.query", "wm_search.retrieve",
    "wm_page.get", "wm_page.list", "wm_page.create", "wm_page.update", "wm_page.delete",
    "wm_page.link", "wm_page.unlink",
    "wm_task.board", "wm_task.list", "wm_task.create", "wm_task.update", "wm_task.delete",
    "wm_graph.stats", "wm_graph.neighbors", "wm_graph.path", "wm_graph.subgraph",
    "wm_memory.list", "wm_memory.get", "wm_memory.add",
    "wm_index.rebuild", "wm_index.status", "wm_index.embed",
    "wm_template.list", "wm_template.get", "wm_template.create",
    "wm_time.start", "wm_time.stop", "wm_time.report",
    "wm_source.add", "wm_source.list", "wm_source.process",
    "wm_lint.check", "wm_validate.check",
    "wm_help", "wm_version",
];

fn register_proxy_handlers(registry: &mut ToolRegistry, server_url: &str) {
    for tool_name in STATIC_TOOLS {
        let url = format!("{server_url}/api/tools/{tool_name}");
        registry.register_proxy(tool_name, move |params| {
            ureq::post(&url).send_json(params)
        });
    }
}
```

### Dev loop

```bash
# Terminal 1: server
cargo run -p wm-server          # starts :4090, opens browser

# Terminal 2: Angular (hot-reload)
cd apps/wm-web && ng serve      # proxies /api → :4090

# Terminal 3: MCP (AI agent)
wm-cli mcp                      # static proxy → :4090
```

All three connect to the same `wm-server` — single EngineState, no duplicates.

## Risks

1. **Concurrent mutation** — page-create → index-rebuild paths need race audit. Read-optimized (arc-swap graph/BM25, RwLock config) but write paths may have assumptions about single-client access.
2. **MCP cold-start** — spawning wm-server adds ~1-2s. Mitigated by parallel spawn + static tools (handshake doesn't wait).
3. **E2E rebasing** — AC-11 sits on Tauri pilot tests. Re-basing onto HTTP is a phase unto itself.
4. **SSE payload size** — graph-refine ships all positions. Fine at 331 nodes, note binary encoding at enterprise scale.
5. **Layout cross-talk** — job-scoped SSE (D7) prevents client A's layout from leaking to client B.

## References
- `apps/wm-web/src/app/services/api.service.ts` — Angular Tauri IPC (to replace)
- `apps/wm-web/src/app/views/graph/graph-view.component.ts` — Graph layout + SSE (to rewrite)
- `apps/wm-web/src-tauri/` — Tauri crate (to delete)
- `apps/wm-core/src/mcp/` — Transport layer (to move to wm-cli)
- `packages/wm-config/src/lib.rs` — Config model for server.json discovery
- `wiki:specs:web-server-build-serve` — Superseded spec
- `wiki:conventions:enterprise-grade` — D1 to rewrite
- `wiki:decisions:axum-over-rocket-for-tower` — Superseded line to annotate
