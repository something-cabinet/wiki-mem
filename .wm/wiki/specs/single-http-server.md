---
title: Single HTTP Server — Replace Tauri with wm-server Daemon
type: spec
status: draft
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

- D1: A single `wm-server` binary owns the **one** `EngineState`. API on `:4090`.
- D2: `wm-cli mcp` becomes a **thin HTTP proxy** — translates MCP/stdio ↔ HTTP/:4090. No in-process EngineState.
- D3: Angular frontend becomes a **pure web app** — replaces `window.__TAURI_INTERNALS__.invoke` with standard `fetch()` to `:4090`.
- D4: **Tauri removed** — `apps/wm-web/src-tauri/` deleted. `wm-server` serves the embedded Angular SPA with `rust-embed`.
- D5: **Singleton daemon** — `wm-server` checks `:4090/health` on startup; if alive, exits (no duplicate). Both `wm-cli mcp` and the user's browser connect to the same process.
- D6: Startup: `wm-server` opens the browser automatically. `wm-cli mcp` spawns `wm-server` if not already running. CLI commands use the server for all read/write operations.

## Requirements

### Functional Requirements

- FR-1: `wm-server` starts, binds to `127.0.0.1:4090`, and responds to `GET /api/health` with `200 OK`
- FR-2: `wm-server` serves the Angular SPA at `GET /` (embedded via `rust-embed`)
- FR-3: All 38+ MCP tools are exposed as RESTful HTTP endpoints under `/api/` (see ARCHITECTURE-SPEC.md §4 for the full route map)
- FR-4: SSE event stream at `GET /api/events` for real-time updates
- FR-5: `wm-cli mcp` starts rmcp on stdio, registers proxy handlers that call `POST http://localhost:4090/api/tools/{name}`
- FR-6: Angular `ApiService` calls `fetch('http://localhost:4090/api/...')` instead of Tauri `invoke()`
- FR-7: `wm-cli <cmd>` operations use HTTP calls to `:4090` where possible (some local-only commands like `wm init` stay local)
- FR-8: `wm-server.exe` opens the default browser on startup (`open::that("http://localhost:4090")`)
- FR-9: `wm-cli mcp` checks `GET /api/health` on startup; if 200, connects as proxy; if not, spawns `wm-server` as a child process

### Non-Functional Requirements

- NFR-1: Localhost HTTP latency <1ms — no perceptible difference from in-process calls
- NFR-2: Build must pass with zero errors
- NFR-3: All existing E2E journeys must pass (adapted for HTTP client)
- NFR-4: Angular SPA must be embeddable in `wm-server` binary (same `build.rs` + `rust-embed` pattern already used for skills)
- NFR-5: Memory footprint drops from ~3× EngineState to 1× (significantly less for CLI and MCP processes)

## Acceptance Criteria

- [ ] AC-1: `cargo run -p wm-server` starts on `:4090`, `GET /api/health` returns `200`
- [ ] AC-2: `GET http://localhost:4090/` serves the Angular app (embedded) — all views render
- [ ] AC-3: `POST /api/search/query { q: "test" }` returns same results as current `wm_search.query` MCP tool
- [ ] AC-4: `wm-cli mcp` starts with `wm-server` down → spawns it → connects → handles MCP tools
- [ ] AC-5: Angular search view returns results via `fetch()` without Tauri
- [ ] AC-6: Angular task board renders with correct data via HTTP
- [ ] AC-7: SSE events deliver real-time updates to Angular (e.g., after page create)
- [ ] AC-8: `wm-cli page list` returns page list via HTTP
- [ ] AC-9: Running `wm-server` a second time detects the existing process and exits cleanly
- [ ] AC-10: Tauri crate (`apps/wm-web/src-tauri/`) can be deleted without breaking the build
- [ ] AC-11: All 14 existing E2E journeys pass with the new HTTP client

## Scenarios

### Scenario 1: User Starts the App
**Given** the user has built the project
**When** they run `cargo run -p wm-server`
**Then** the server starts on `127.0.0.1:4090`
**And** the default browser opens to `http://localhost:4090`
**And** the Angular app loads with all views functional

### Scenario 2: AI Agent Connects via MCP
**Given** an AI agent (OpenCode) launches `wm mcp`
**When** `wm-cli mcp` checks `GET /api/health`
**If** `:4090` responds → connects as proxy
**If** `:4090` is down → spawns `wm-server`, waits for health, then connects
**Then** the agent uses all 38+ tools via MCP protocol, each proxied to the same server

### Scenario 3: Concurrent Access
**Given** the user is browsing the Angular UI (connected to `:4090`)
**When** an AI agent makes a mutation via MCP (e.g., creates a page)
**Then** the change is reflected in the same `EngineState`
**And** the Angular UI receives an SSE event
**And** the new page appears in the UI without manual refresh

### Scenario 4: CLI Command
**Given** a developer runs `wm-cli task list`
**When** the CLI connects to `localhost:4090`
**Then** it fetches tasks via `GET /api/tasks`
**And** displays them (same Ratatui TUI, but backed by HTTP)

## Technical Notes

### New crate structure

```
apps/wm-server/           # NEW: HTTP server + engine owner
├── Cargo.toml            # deps: wm-engine, axum, tower-http, tokio, serde, open, rust-embed
├── build.rs              # embed Angular dist/ (same pattern as current wm-web)
└── src/
    ├── main.rs           # entry: parse args, health check, start axum, open browser
    ├── router.rs         # all route definitions
    ├── state.rs          # AppState (Arc<EngineState>)
    └── routes/           # per-domain route modules
```

### Health check / singleton logic

```rust
// wm-server main.rs
async fn try_bind(port: u16) -> Result<TcpListener> {
    match TcpListener::bind(("127.0.0.1", port)).await {
        Ok(listener) => Ok(listener),  // first instance
        Err(_) => {
            // Port in use — check if it's us
            if check_health(port).await {
                eprintln!("wm-server already running on :{port}");
                std::process::exit(0);  // graceful exit, not an error
            }
            // Port taken by something else — error
            bail!("Port {port} in use by non-wm-server process");
        }
    }
}
```

### MCP proxy logic

```rust
// apps/wm-cli/src/mcp_proxy.rs
fn start_mcp_proxy() {
    let server_url = "http://localhost:4090";

    // Ensure server is running
    if !check_health(server_url) {
        spawn_wm_server();
        wait_for_health(server_url, Duration::from_secs(10))?;
    }

    // Discover tools from server
    let tools = fetch_tool_list(server_url)?;

    // Register proxy handlers
    for tool in tools {
        registry.register_proxy(tool.name, move |params| {
            http_client.post(format!("{server_url}/api/tools/{}", tool.name))
                .json(&params)
                .send()?
                .json()
        });
    }

    serve_rmcp(registry)?;  // same as today
}
```

### Angular HTTP client

Replace in `api.service.ts`:

```typescript
// Before (Tauri)
private async tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return await window.__TAURI_INTERNALS__.invoke(cmd, args);
}

// After (HTTP)
private async httpCall<T>(domain: string, action: string, body?: unknown): Promise<T> {
  const res = await fetch(`http://localhost:4090/api/${domain}/${action}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: body ? JSON.stringify(body) : undefined,
  });
  const json = await res.json();
  if (!json.success) throw new Error(json.error?.message ?? 'Unknown error');
  return json.data;
}
```

### Route design (matching MCP tools)

| Method | Path | MCP Tool Equivalent |
|--------|------|---------------------|
| POST | `/api/tools/wm_search.query` | Direct tool dispatch |
| GET | `/api/health` | — |
| GET | `/api/initial` | `wm_initial` |
| POST | `/api/search/query` | `wm_search.query` |
| GET | `/api/pages` | `wm_page.list` |
| ... | (full map in ARCHITECTURE-SPEC.md §4) | |

### Angular dev proxy

During development, `ng serve` already has `proxy.conf.json`. Update to proxy `/api` to `http://localhost:4090`:

```json
{
  "/api": {
    "target": "http://localhost:4090",
    "secure": false
  }
}
```

This allows `ng serve` + `wm-server` to coexist during migration.

## Migration Phases

### Phase 1: Create wm-server (alongside existing stack)
1. `cargo init apps/wm-server`
2. Add axum, tower-http, wm-engine deps
3. Implement router with full ~81 route surface
4. Embed Angular SPA via `rust-embed`
5. Verify: server starts, API works, Angular renders in browser
6. Existing Tauri + MCP continue working unchanged

### Phase 2: Wire Angular to HTTP
1. Update `proxy.conf.json` for dev
2. Rewrite `api.service.ts` to use `fetch()` with HTTP fallback
3. Remove Tauri `invoke()` dependency
4. Verify: `ng serve` + `wm-server` together

### Phase 3: Thin MCP proxy
1. Add `reqwest` to `wm-cli`
2. Create `mcp_proxy.rs` — dynamic tool discovery + HTTP dispatch
3. `wm-cli mcp` health-checks `:4090`, spawns server if needed
4. Remove `wm-core::mcp` module (cleanup)

### Phase 4: Remove Tauri
1. Delete `apps/wm-web/src-tauri/`
2. Remove `tauri` deps from workspace
3. Remove `apps/wm-web` from Cargo workspace members (or keep as path dep for embedding)
4. Verify: `cargo build` passes — zero Tauri references remain

### Phase 5: CLI migration
1. Migrate `wm-cli` commands to use HTTP where applicable
2. Keep local-only commands (`wm init`, `wm setup`)
3. Ratatui TUI connects to HTTP engine

## Open Questions

- [ ] Should `wm-server` require a password/token for remote access, or stay localhost-only initially?
- [ ] Should `wm-cli mcp` embed the server start inline (same process, different thread) or spawn a child process?
- [ ] Migration order: do we ship the server first, then migrate MCP, or ship both at once with a feature flag?

## Related Specs

- [ARCHITECTURE-SPEC.md](../../ARCHITECTURE-SPEC.md) — Root-level architecture specification with full route map and migration phases (the canonical reference)
- [Enterprise-Grade Architecture](../conventions/enterprise-grade.md) — D1 (Tauri primary) is overridden by this spec
