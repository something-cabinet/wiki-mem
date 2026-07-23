---
title: Phase 1 — Create wm-server crate (Axum HTTP daemon)
type: task
status: todo
priority: high
tags: [architecture, server, http, phase-1]
spec: specs/single-http-server
relates_to:
  - {type: implements, target: wiki:specs:single-http-server}
---

## Overview

Create the `apps/wm-server` crate — a standalone Axum HTTP server that owns the single `EngineState`, serves the Angular SPA, and exposes the full REST API on a dynamic port (defaulting to 4090).

This is the first step toward replacing Tauri entirely. The existing Tauri + in-process MCP stack continues working unchanged during this phase.

## Acceptance Criteria

- [ ] `apps/wm-server/` crate exists with `Cargo.toml`, `main.rs`, `router.rs`, `state.rs`
- [ ] Dependencies: `wm-engine`, `axum`, `tower-http`, `tokio`, `serde`, `serde_json`, `rust-embed`, `open`
- [ ] `GET /api/health` returns `200 OK`
- [ ] `GET /api/initial` returns engine state (graph node/edge count, uptime, etc.)
- [ ] `POST /api/search/query` with `{"q": "test"}` returns same results as current MCP `wm_search.query` tool
- [ ] `GET /api/pages` returns page list (same as `wm_page.list`)
- [ ] `GET /` serves the Angular SPA (embedded via `rust-embed` from `apps/wm-web/dist/`)
- [ ] Server writes `.wm/server.json` with `{ port, pid, started_at }` on startup
- [ ] Server reads `.wm/server.json` on startup — if alive, exits gracefully (singleton)
- [ ] Server opens default browser to `http://localhost:{port}` on startup
- [ ] `cargo build` passes — zero Tauri references in wm-server

## Implementation Notes

See `specs/single-http-server.md` for full architecture. Route map in `ARCHITECTURE-SPEC.md` §4.

Start with these routes (the 13 that already exist as Tauri commands):
- `/api/health` (new)
- `/api/initial` (from `commands.rs:get_initial`)
- `/api/search/query` (from `commands.rs:search`)
- `/api/pages` (CRUD — from Tauri page commands)
- `/api/tasks/board` (from Tauri task commands)
- `/api/graph/stats`, `/api/graph/neighbors/{id}` (from Tauri graph commands)
- `/api/memory/list` (from Tauri memory commands)
- `/api/events` (SSE — from Tauri event system)

The remaining ~68 routes (ARCHITECTURE-SPEC.md §4) can be added incrementally in follow-up tasks.

## Related Specs

- `specs/single-http-server` — Full architecture spec
- `ARCHITECTURE-SPEC.md` — Root-level architecture with full route map
