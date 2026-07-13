# WM Architecture Redesign: Single Engine, Thin MCP Proxy

**Status:** Specification  
**Date:** 2026-07-13  
**Confidence:** High

---

## 1. Summary

### 1.1 Problem

Three processes currently each own a full `EngineState` (graph, BM25, embedder, memory). MCP tools call in-process functions directly. The HTTP server (bundled inside `wm-web`) has only 13 routes covering 5 of 19 tool domains. There is no single source of truth for wiki state.

### 1.2 Target Architecture

```
                  ┌─────────────────────────────┐
                  │        wm-server             │
                  │   (single engine process)     │
                  │   axum HTTP + JSON API        │
                  │   owns: graph, BM25, embedder │
                  │   all disk writes route here  │
                  └──────────┬───────────────────┘
                             │  HTTP :4090
                ┌────────────┼────────────────┐
                │            │                │
        ┌───────▼──────┐ ┌──▼──────────┐ ┌──▼──────────┐
        │   wm-cli mcp  │ │ wm-web (ui) │ │ wm-cli cmds │
        │  (thin proxy) │ │ (Angular)   │ │  (CLI tools) │
        │  rmcp + HTTP  │ │ dev proxy   │ │  HTTP calls  │
        │  calls server │ │ to wm-srv   │ │  to wm-srv   │
        └───────────────┘ └─────────────┘ └──────────────┘
```

- **One** process owns the engine: `wm-server` (or `wm-cli serve`)
- **Everyone else** talks to it via HTTP (JSON REST API)
- **MCP becomes a thin proxy** — `wm-cli mcp` starts the rmcp stdio transport, but each tool handler calls `POST /api/tools/wm_search.query` instead of running in-process
- **Angular UI** serves static files only (dev: `ng serve` with proxy; prod: served by wm-server or a static file server)

### 1.3 Design Decision: Server Location

**Recommendation: New `apps/wm-server` crate** (option A), with `wm-cli serve` as a convenience alias that embeds and spawns the server.

| Option | Pros | Cons |
|--------|------|------|
| A. `apps/wm-server` (recommended) | Clean separation, independent deployable, clear ownership | One more crate |
| B. `wm-cli serve` subcommand | Fewer crates, simpler discovery | `wm-cli` binary gets big, can't run server without CLI binary |
| C. Part of `wm-core` | Zero new crates | Violates library-only role of wm-core; embeds axum in a lib |

**Why A over B:** `wm-server` is independently useful (you may want to run it as a daemon/systemd service without the CLI). `wm-cli` can still have `wm-cli serve` that spawns or proxies to the server process for user convenience.

## 2. New Directory Tree

```
vpp-rag/
├── .wm/                          # wiki data dir (unchanged, stays at repo root)
├── apps/
│   ├── wm-core/                  # Library: graph, search, BM25, embedder, skill engine, config
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── config.rs
│   │   │   ├── embed/
│   │   │   ├── engine/           # EngineState + MainEngine (NO mcp/ here anymore)
│   │   │   ├── graph.rs
│   │   │   ├── page.rs
│   │   │   ├── parser.rs
│   │   │   ├── reference.rs
│   │   │   ├── search/
│   │   │   ├── skill.rs
│   │   │   ├── source.rs
│   │   │   ├── task.rs
│   │   │   ├── template_engine.rs
│   │   │   ├── util.rs
│   │   │   ├── code_intel.rs
│   │   │   ├── onnx.rs
│   │   │   ├── error.rs
│   │   │   └── status.rs
│   │   └── tests/
│   │
│   ├── wm-server/                # NEW: HTTP server + engine owner
│   │   ├── Cargo.toml            # dep: wm-core, axum, tower-http, tokio, serde*
│   │   └── src/
│   │       ├── main.rs           # entry: parse args, create engine, start axum
│   │       ├── router.rs         # all route definitions
│   │       ├── state.rs          # AppState (Arc<EngineState>)
│   │       └── routes/
│   │           ├── mod.rs
│   │           ├── initial.rs
│   │           ├── search.rs
│   │           ├── pages.rs
│   │           ├── tasks.rs
│   │           ├── graph.rs
│   │           ├── memory.rs
│   │           ├── source.rs     # NEW
│   │           ├── time.rs       # NEW
│   │           ├── index.rs      # NEW
│   │           ├── model.rs      # NEW
│   │           ├── lint.rs       # NEW
│   │           ├── validate.rs   # NEW
│   │           ├── log.rs        # NEW
│   │           ├── project.rs    # NEW
│   │           ├── skills.rs     # NEW
│   │           ├── reference.rs  # NEW
│   │           ├── decision.rs   # NEW
│   │           ├── doc.rs        # NEW
│   │           ├── template.rs   # NEW
│   │           ├── code.rs       # NEW
│   │           ├── events.rs     # SSE
│   │           └── health.rs
│   │
│   ├── wm-cli/                   # CLI binary + MCP proxy (gets thinner)
│   │   ├── Cargo.toml            # dep: wm-core (for config), clap, tokio, reqwest, rmcp
│   │   └── src/
│   │       ├── main.rs           # CLI commands (most become HTTP calls, some stay local)
│   │       ├── mcp_proxy.rs      # NEW: thin MCP proxy — tool handlers that call HTTP
│   │       └── tui.rs
│   │
│   └── wm-web/                   # Angular UI only (no Rust backend)
│       ├── package.json
│       ├── angular.json
│       ├── proxy.conf.json       # proxy /api → http://localhost:4090
│       └── src/
│
├── Cargo.toml                    # workspace = ["apps/wm-core", "apps/wm-server", "apps/wm-cli"]
├── WIKI-MEM.md
├── AGENTS.md
└── ...
```

### Migration Notes

1. `wm-core/src/mcp/` **moves** to `apps/wm-cli/src/mcp_tools/` (or stays, but loses `EngineState` dependency — becomes HTTP-calling closures)
2. `wm-web/src/api/` **moves** to `apps/wm-server/src/routes/` (unified under the server)
3. `wm-web/src/lib.rs` (run_server, static file serving) can be deprecated or moved to wm-server
4. `.wm/` directory stays at repo root (constraint satisfied)

## 3. Crate Responsibilities

### 3.1 `apps/wm-core` (library — minimal change)

| Was | Becomes |
|-----|---------|
| Owns MCP tools | **Removes** MCP module entirely (or keeps only types, not handlers) |
| Owns EngineState | Stays — this is the engine |
| Search, graph, BM25 | Unchanged |
| Embedder, ONNX | Unchanged |
| Config, parser, error | Unchanged |

**Key change:** The `mcp/` module is removed from `wm-core`. It had no business being in a library — MCP tools are a transport concern. The typed tool registration (TypedRegister trait) stays if useful for server-side route implementations, but likely the server just calls `wm_core::page::get_page()` and similar functions directly, wrapping results in JSON responses — same pattern as the current HTTP routes.

**Actually simpler:** The server routes can call `wm_core::engine` functions directly (just like current `wm-web/src/api/pages.rs` calls `wm_core::page::list_pages`). No need for the MCP tool abstraction layer at all. The thin MCP proxy in wm-cli will call HTTP endpoints that mirror the core function signatures.

### 3.2 `apps/wm-server` (new — engine owner)

- Creates, owns, and manages the single `EngineState`
- Exposes ~75+ HTTP endpoints (one per tool domain, see §4)
- Background tasks: graph rebuild loop, audit log consumer, index scheduler
- SSE event stream for real-time updates
- Optional: serve Angular static files in production mode
- CLI flags: `--port`, `--project`, `--host`

### 3.3 `apps/wm-cli` (CLI + thin MCP proxy)

- **Thin MCP proxy:** `wm-cli mcp` starts rmcp stdio transport, but handlers call `POST http://localhost:4090/api/tools/{tool_name}` instead of in-process functions
- **CLI commands:** Most become HTTP calls. Some stay local (`wm init`, `wm setup`, `wm agents`)
- **TUI:** Remains local (connects to server via HTTP)
- **Web command:** `wm-cli web` either spawns `wm-server` as a subprocess or just tells the user to run `wm-server` directly
- Can optionally auto-start wm-server if not running (nice UX)

### 3.4 `apps/wm-web` (Angular UI only)

- Pure Angular application
- No Rust code (no Cargo.toml in workspace)
- `ng serve` with proxy.conf to `localhost:4090`
- Production: built to `dist/`, served by wm-server as static files (or nginx, or CDN)
- **Or** keep it in the Cargo workspace with `build.rs` for embedding — if we want `wm-server` to serve the UI in a single binary. Recommendation: keep embedding in wm-server for single-binary deployment.

## 4. HTTP API Surface

### 4.1 Route Design

Pattern: `GET/POST /api/{domain}/{action}` with JSON request/response bodies.

**POST vs GET:** Use POST for read queries that have JSON bodies (search, retrieve, resolve). Use GET for simple id-based reads. Use POST/PUT/DELETE for mutations.

### 4.2 Complete Endpoint Map

The server needs to expose all operations that MCP tools currently provide. Here's the full map, organized by domain:

#### Initial / Health

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| GET | `/api/health` | — | ✅ exists |
| GET | `/api/initial` | `wm_initial` | ✅ exists |
| GET | `/api/help` | `wm_help` | ❌ missing |
| GET | `/api/project/status` | `wm_project.status` | ❌ missing |
| POST | `/api/project/detect` | `wm_project.detect` | ❌ missing |
| POST | `/api/project/set` | `wm_project.set` | ❌ missing |

#### Search

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| POST | `/api/search/query` | `wm_search.query` | ✅ exists (as GET /api/search) |
| POST | `/api/search/retrieve` | `wm_search.retrieve` | ❌ missing |
| POST | `/api/search/resolve` | `wm_search.resolve` | ❌ missing |

#### Pages

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| GET | `/api/pages` | `wm_page.list` | ✅ exists |
| GET | `/api/pages/{id}` | `wm_page.get` | ✅ exists |
| POST | `/api/pages` | `wm_page.create` | ✅ exists |
| PUT | `/api/pages/{id}` | `wm_page.update` | ✅ exists |
| DELETE | `/api/pages/{id}` | `wm_page.delete` | ✅ exists |
| POST | `/api/pages/link` | `wm_page.link` | ❌ missing |
| POST | `/api/pages/unlink` | `wm_page.unlink` | ❌ missing |

#### Tasks

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| POST | `/api/tasks` | `wm_task.create` | ❌ missing |
| GET | `/api/tasks/{id}` | `wm_task.get` | ❌ missing |
| PUT | `/api/tasks/{id}` | `wm_task.update` | ❌ missing |
| DELETE | `/api/tasks/{id}` | `wm_task.delete` | ❌ missing |
| GET | `/api/tasks` | `wm_task.list` | ❌ missing |
| GET | `/api/tasks/board` | `wm_task.board` | ✅ exists |
| POST | `/api/tasks/{id}/check-ac` | `wm_task.check_ac` | ❌ missing |
| POST | `/api/tasks/{id}/uncheck-ac` | `wm_task.uncheck_ac` | ❌ missing |
| POST | `/api/tasks/{id}/subtask` | `wm_task.subtask` | ❌ missing |

#### Graph

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| GET | `/api/graph/stats` | `wm_graph.stats` | ✅ exists |
| GET | `/api/graph/neighbors/{id}` | `wm_graph.neighbors` | ✅ exists |
| POST | `/api/graph/subgraph` | `wm_graph.subgraph` | ❌ missing |
| POST | `/api/graph/path` | `wm_graph.path` | ❌ missing |

#### Memory

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| GET | `/api/memory` | `wm_memory.list` | ✅ exists (partial — no layer filter) |
| GET | `/api/memory/{id}` | `wm_memory.get` | ❌ missing |
| POST | `/api/memory` | `wm_memory.add` | ❌ missing |
| PUT | `/api/memory/{id}` | `wm_memory.update` | ❌ missing |
| DELETE | `/api/memory/{id}` | `wm_memory.delete` | ❌ missing |
| POST | `/api/memory/{id}/promote` | `wm_memory.promote` | ❌ missing |

#### Sources

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| POST | `/api/sources` | `wm_source.add` | ❌ missing |
| POST | `/api/sources/{id}/process` | `wm_source.process` | ❌ missing |
| POST | `/api/sources/{id}/complete` | `wm_source.complete` | ❌ missing |
| POST | `/api/sources/{id}/error` | `wm_source.error` | ❌ missing |
| GET | `/api/sources` | `wm_source.list` | ❌ missing |
| POST | `/api/sources/{id}/verify` | `wm_source.verify` | ❌ missing |
| POST | `/api/sources/discover` | `wm_source.discover` | ❌ missing |
| DELETE | `/api/sources/{id}` | `wm_source.remove` | ❌ missing |
| GET | `/api/sources/{id}` | `wm_source.status` | ❌ missing |

#### Time

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| POST | `/api/time/start` | `wm_time.start` | ❌ missing |
| POST | `/api/time/stop` | `wm_time.stop` | ❌ missing |
| POST | `/api/time/add` | `wm_time.add` | ❌ missing |
| GET | `/api/time/report` | `wm_time.report` | ❌ missing |

#### Index

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| POST | `/api/index/rebuild` | `wm_index.rebuild` | ❌ missing |
| POST | `/api/index/embed` | `wm_index.embed` | ❌ missing |
| GET | `/api/index/status` | `wm_index.status` | ❌ missing |

#### Model

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| GET | `/api/models` | `wm_model.list` | ❌ missing |
| GET | `/api/models/status` | `wm_model.status` | ❌ missing |
| POST | `/api/models/download` | `wm_model.download` | ❌ missing |
| DELETE | `/api/models/{name}` | `wm_model.remove` | ❌ missing |

#### Lint & Validate

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| GET | `/api/lint/check` | `wm_lint.check` | ❌ missing |
| POST | `/api/lint/fix` | `wm_lint.fix` | ❌ missing |
| POST | `/api/validate/check` | `wm_validate.check` | ❌ missing |

#### Logs

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| GET | `/api/logs/recent` | `wm_log.recent` | ❌ missing |
| GET | `/api/logs/since` | `wm_log.since` | ❌ missing |
| GET | `/api/logs/filter` | `wm_log.filter` | ❌ missing |

#### Skills

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| POST | `/api/skills/trigger` | `wm_skill.trigger` | ❌ missing |

#### References

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| POST | `/api/refs/extract` | `wm_ref.extract` | ❌ missing |
| POST | `/api/refs/resolve` | `wm_ref.resolve` | ❌ missing |
| POST | `/api/refs/resolve-all` | `wm_ref.resolve_all` | ❌ missing |

#### Decisions

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| POST | `/api/decisions` | `wm_decision.create` | ❌ missing |
| GET | `/api/decisions/{id}` | `wm_decision.get` | ❌ missing |

#### Docs

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| GET | `/api/docs` | `wm_doc.list` | ❌ missing |
| GET | `/api/docs/{path}` | `wm_doc.get` | ❌ missing |
| POST | `/api/docs` | `wm_doc.create` | ❌ missing |
| PUT | `/api/docs/{path}` | `wm_doc.update` | ❌ missing |
| DELETE | `/api/docs/{path}` | `wm_doc.delete` | ❌ missing |

#### Templates

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| GET | `/api/templates` | `wm_template.list` | ❌ missing |
| GET | `/api/templates/{name}` | `wm_template.get` | ❌ missing |
| POST | `/api/templates` | `wm_template.create` | ❌ missing |
| POST | `/api/templates/{name}/run` | `wm_template.run` | ❌ missing |

#### Code Intelligence

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| POST | `/api/code/search` | `wm_code.search` | ❌ missing |
| POST | `/api/code/symbols` | `wm_code.symbols` | ❌ missing |
| POST | `/api/code/deps` | `wm_code.deps` | ❌ missing |

#### Events (SSE)

| Method | Path | MCP Tool | Status |
|--------|------|----------|--------|
| GET | `/api/events` | — | ✅ exists |

**Totals:**
- **Existing:** 13 routes (health, initial, search(query), pages CRUD + list, tasks/board, graph/stats, graph/neighbors, memory/list, events)
- **Missing:** ~68 routes across 14 new domains
- **Full API:** ~81 routes across 19 domains

### 4.3 JSON Protocol

All responses follow a consistent envelope (where appropriate):

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

Or for errors:

```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Page 'tasks/auth' not found"
  }
}
```

The existing API already follows this pattern (see `wm-web/src/api/pages.rs`). Standardize it across all new routes.

### 4.4 Route Organization in Code

```
apps/wm-server/src/routes/
├── mod.rs            # re-exports + Router builder helper
├── initial.rs        # /health, /initial, /help
├── search.rs         # /api/search/*
├── pages.rs          # /api/pages/*
├── tasks.rs          # /api/tasks/*
├── graph.rs          # /api/graph/*
├── memory.rs         # /api/memory/*
├── source.rs         # /api/sources/*
├── time.rs           # /api/time/*
├── index.rs          # /api/index/*
├── model.rs          # /api/models/*
├── lint.rs           # /api/lint/*
├── validate.rs       # /api/validate/*
├── log.rs            # /api/logs/*
├── project.rs        # /api/project/*
├── skills.rs         # /api/skills/*
├── reference.rs      # /api/refs/*
├── decision.rs       # /api/decisions/*
├── doc.rs            # /api/docs/*
├── template.rs       # /api/templates/*
├── code.rs           # /api/code/*
└── events.rs         # /api/events (SSE)
```

## 5. MCP → HTTP Proxy Pattern

### 5.1 Design

The thin MCP proxy in `wm-cli` does the following:

1. Starts rmcp server on stdio (same as today)
2. Each registered tool handler calls a local HTTP server instead of in-process functions
3. The `EngineState` is **not** created in the CLI process

### 5.2 Example: `wm_search.query` Proxy

**Before (current, in-process):**
```rust
// wm-core/src/mcp/tools/search.rs
let e = engine.clone();
registry.register_read("wm_search.query", "Search the wiki...", move |input: WmSearchQueryInput| {
    let params = crate::search::QueryParams { ... };
    let results = crate::search::run_unified_search(&e, &params)?;
    Ok(serde_json::json!({ "results": ... }))
});
```

**After (thin proxy, HTTP):**
```rust
// apps/wm-cli/src/mcp_proxy.rs
use reqwest::Client;

fn register_search_proxy(registry: &mut ToolRegistry, client: &Client, base_url: &str) {
    let url = format!("{}/api/search/query", base_url);
    let c = client.clone();

    registry.register_read(
        "wm_search.query",
        "Search the wiki and/or memory (keyword/semantic/hybrid)",
        move |input: WmSearchQueryInput| {
            let rt = tokio::runtime::Handle::current();
            let resp = rt.block_on(
                c.post(&url)
                    .json(&serde_json::to_value(input)?)
                    .send()
            ).map_err(|e| ToolError::internal(format!("HTTP error: {e}")))?;

            let body: serde_json::Value = rt.block_on(resp.json())
                .map_err(|e| ToolError::internal(format!("JSON parse error: {e}")))?;

            Ok(body)
        },
    );
}
```

### 5.3 Benefits

- **Zero engine state in CLI process** — memory footprint drops dramatically
- **Single engine instance** — no stale graph problems
- **Auto-discovery** — proxy can call `/api/help` on startup to dynamically register tools (the server knows all available tools)
- **Health check** — proxy checks server health before announcing readiness
- **Graceful degradation** — if server is down, proxy returns clear error messages

### 5.4 Dynamic Tool Registration

The proxy can auto-discover tools from the server:

```rust
fn register_all_tools(registry: &mut ToolRegistry, client: &Client, base_url: &str) {
    let tools = client.get(format!("{}/api/help", base_url))
        .send()?.json::<Value>()?;

    for tool in tools["available_tools"].as_array()? {
        let name = tool["name"].as_str()?;
        let desc = tool["description"].as_str()?;
        // Register a generic proxy handler for each tool
        register_proxy_handler(registry, client, base_url, name, desc);
    }
}
```

This eliminates the need for `register_all_tools()` in core — the server becomes the definitive source of available tools.

### 5.5 Transport Options

The proxy uses `reqwest::blocking` for simplicity, since rmcp handlers are sync closures. Alternatively, use `tokio::task::block_in_place` + async reqwest:

```rust
// Handle async HTTP inside sync rmcp handler
let rt = tokio::runtime::Handle::current();
let body = rt.block_on(async {
    client.post(&url).json(&params).send().await?.json::<Value>().await
})?;
```

This is fine for MCP — MCP tools are called sequentially by a single client, so blocking the thread briefly is acceptable.

## 6. Angular UI Communication

### 6.1 Development

```bash
cd apps/wm-web
ng serve --proxy-config proxy.conf.json
```

`proxy.conf.json` already proxies `/api` to `localhost:3000`. Update to `localhost:4090` (or whatever port wm-server uses).

### 6.2 Production

Two options:

**Option 1 (recommended):** `wm-server` serves static files. Keep `build.rs` + `rust-embed` pattern from current `wm-web`. The Angular `dist/` gets embedded in the server binary. Single binary deployment.

**Option 2:** Separate deployment. `nginx` serves Angular files, proxies `/api` to `wm-server`. More scalable, but two processes.

For now, option 1 keeps the single-binary simplicity of the current setup.

### 6.3 What Moves Where

| Current | Destination |
|---------|-------------|
| `wm-web/ui/` → Angular source | `apps/wm-web/` (standalone Angular project) |
| `wm-web/src/lib.rs` (run_server, static serving) | `apps/wm-server/src/` (merged into server) |
| `wm-web/src/api/*.rs` (route handlers) | `apps/wm-server/src/routes/*.rs` |
| `wm-web/build.rs` (embed Angular dist) | Moves to `apps/wm-server/build.rs` |

## 7. Incremental Migration Path

### Phase 1: Restructure (no behavior change)

1. Create `apps/` directory
2. `git mv wm-core → apps/wm-core`
3. `git mv wm-cli → apps/wm-cli`
4. `git mv wm-web → apps/wm-web`
5. Update `Cargo.toml` workspace paths: `members = ["apps/wm-core", "apps/wm-cli", "apps/wm-web"]`
6. Update internal path deps: `path = "../wm-core"` → `path = "../wm-core"` (no change, they're siblings)
7. **Verify:** `cargo build` passes

### Phase 2: Create wm-server (new crate, no impact on existing)

1. `cargo init apps/wm-server --lib` (then make it a bin)
2. Add `apps/wm-server/Cargo.toml` with deps: `wm-core = { path = "../wm-core" }`, axum, tower-http, serde, etc.
3. Copy `wm-web/src/api/` → `apps/wm-server/src/routes/` (don't remove original yet)
4. Implement new server binary with full route set (all ~81 routes from §4)
5. **Verify:** Server starts, serves health check, runs alongside existing `wm-cli web`

### Phase 3: Migrate wm-web to Angular-only

1. `apps/wm-web/Cargo.toml` — remove or gut (only build.rs + embedding stays)
2. Move embedding + static serving to `apps/wm-server/`
3. Remove `wm-web` from Cargo workspace members (or keep as path dep for embedding)
4. Update Angular proxy to point at wm-server port
5. `wm-cli web` command: starts wm-server as subprocess + opens browser (or just tells user)

### Phase 4: Thin MCP Proxy

1. Add `reqwest` to `wm-cli/Cargo.toml`
2. Create `apps/wm-cli/src/mcp_proxy.rs` with generic HTTP-calling tool handlers
3. Auto-discover tools from server on startup
4. `wm-cli mcp` first checks server health; if down, either starts server or errors
5. Remove `wm-core/src/mcp/` module (cleanup)

### Phase 5: Cleanup

1. Remove `EngineState` dependency from `wm-cli` (except for TUI, which can talk HTTP too)
2. Remove deprecated `wm-web/src/lib.rs` server code
3. Remove `wm-web` from workspace entirely if fully Angular-only
4. Update documentation

### Backward Compatibility During Migration

- `wm-cli mcp` still works (first via in-process engine, later via proxy)
- `wm-cli web` still works (runs embedded server or spawns wm-server)
- `.wm/` data directory unchanged
- All MCP tool names unchanged

## 8. Risk Analysis

| Risk | Mitigation |
|------|-----------|
| HTTP latency for MCP tools | localhost HTTP is <1ms — negligible vs. ONNX/BM25 compute time |
| Server crash takes down everything | Single process is simpler; systemd/process manager for restart |
| wm-cli needs server running | Auto-start or clear error: "Server not running. Run `wm-cli serve`" |
| Migration breaks existing MCP clients | Phase-gated: old in-process path works until proxy is stable |
| Angular proxy config change | Minor — one-line `proxy.conf.json` update |

## 9. Implementation Order

1. **Create `ARCHITECTURE-SPEC.md`** (this document) — ✅ done
2. **Phase 1: Restructure** — `git mv` crates under `apps/`, verify build
3. **Phase 2: Create wm-server** — implement full REST API, verify all routes
4. **Phase 3: Migrate wm-web to Angular-only** — remove Rust backend from web crate
5. **Phase 4: Thin MCP proxy** — HTTP-calling tool registry in wm-cli
6. **Phase 5: Cleanup** — remove deprecated code, update docs

---

*End of specification. For questions, see `C:\Users\hk\.kimaki\projects\vpp-rag\WIKI-MEM.md`*
