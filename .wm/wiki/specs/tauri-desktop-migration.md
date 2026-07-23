---
title: Tauri Desktop App — Migration
type: spec
tags: [spec, tauri, desktop, migration, enterprise]
status: draft
relates_to:
  - {type: references, target: wiki:tasks:srv-wire-angular-to-http--replace-tauri-ipc-with-fetch}
---

## Overview

Migrate from the current HTTP-server-based architecture (`wm-server` + Angular dev server) to a **Tauri v2 desktop app** where the Angular frontend is bundled natively and calls Rust backend functions directly via IPC, not HTTP. This eliminates `wm-server` entirely.

## Motivation

The enterprise-grade architecture commits to Tauri as the primary deployment. All API routes currently served by `wm-server` (Axum) become Tauri commands. The Angular frontend is statically bundled by Tauri — no separate server, no CORS, no port management.

## Locked Decisions

- D1: Tauri v2 is the primary deployment. All-in, no fallback to `wm-server`.
- D2: `wm-server` crate is removed. All routes become `#[tauri::command]` functions.
- D3: `wm-cli mcp` stays for OpenCode AI agent integration (separate binary).
- D4: `wm-core` is imported directly by the Tauri backend (no HTTP, no proxy).
- D5: Angular frontend (`wm-web`) is bundled statically by Tauri, not served by a dev server.
- D6: The `graph_rebuild_loop` background task runs in Tauri's `setup()` via `tokio::spawn`.

## Requirements

### Functional Requirements

- FR-1: Create `apps/wm-tauri/` as a new Cargo workspace member that depends on `wm-core`
- FR-2: All current `wm-server` API routes become `#[tauri::command]` functions (search, pages CRUD, tasks board, graph stats/neighbors/full, memory list, initial state)
- FR-3: The Angular `ApiService` switches from `HttpClient` + `Observable` to `invoke()` + `Promise` (or wraps in `from()` for backward compat)
- FR-4: SSE events stream (`GET /api/events`) becomes Tauri events (`emit()`/`listen()`)
- FR-5: SPA static file serving becomes Tauri's native frontend bundling (no `handle_spa` handler needed)
- FR-6: The mock server (`packages/wm-mock-server`) and E2E tests (`apps/wm-web-e2e`) remain unchanged
- FR-7: `wm-server` crate is deleted after migration

### Non-Functional Requirements

- NFR-1: All existing functionality must work identically in Tauri (search, pages, tasks, graph, memory, settings)
- NFR-2: Build pipeline: `cargo build -p wm-tauri` produces a standalone desktop binary
- NFR-3: Development workflow: Angular dev server + `tauri dev` for hot-reload
- NFR-4: No regressions in `wm-cli mcp` (unaffected by this change)

## Acceptance Criteria

- [ ] AC-1: `tauri dev` launches the Angular frontend with hot-reload and all API calls work via IPC
- [ ] AC-2: `cargo build -p wm-tauri` compiles and produces a binary that bundles the frontend
- [ ] AC-3: All 6 views (Search, Graph, Tasks, Pages, Memory, Settings) load and function correctly in the Tauri window
- [ ] AC-4: Page CRUD (list, get, create) works via Tauri IPC
- [ ] AC-5: Search returns results via Tauri IPC
- [ ] AC-6: Task board loads via Tauri IPC
- [ ] AC-7: Graph stats and neighbors work via Tauri IPC
- [ ] AC-8: Memory list and create work via Tauri IPC
- [ ] AC-9: `wm-server` crate is deleted with no dangling references
- [ ] AC-10: `wm-cli mcp` still compiles and works independently
- [ ] AC-11: All 14 E2E journeys still pass (against mock server)
- [ ] AC-12: System tray icon, window management, and native menus work (if implemented)

## Scenarios

### Scenario 1: Fresh Tauri Launch
**Given** a user on a clean install
**When** they run the Tauri desktop app
**Then** the Angular frontend loads in a native window
**And** the graph rebuilds from `.wm/wiki/` on startup
**And** all data operations go through Tauri IPC (no HTTP)

### Scenario 2: Search via IPC
**Given** the Tauri app is running
**When** a user types a query and clicks Search
**Then** the Angular frontend calls `invoke('search', { query })`
**And** the Rust backend runs the search via `wm_core::search`
**And** results are returned directly (no HTTP round-trip)

### Scenario 3: Graph Full Dump
**Given** the Tauri app is running
**When** a user opens the Graph view
**Then** the Angular frontend calls `invoke('graph_full', {})`
**And** the Rust backend serializes the graph snapshot and returns nodes + edges
**And** positions are computed on a background thread

### Scenario 4: Background Rebuild
**Given** the Tauri app is running
**When** a page is created or edited
**Then** the `stale_flag` is set in the engine
**And** the background `graph_rebuild_loop` picks it up within 10 seconds
**And** the frontend is notified via a Tauri event

## Implementation Phases

### Phase 1: Tauri Scaffold + Core Commands

1. Install Tauri CLI: `npm install -D @tauri-apps/cli`
2. `npx tauri init` in `apps/wm-web/` → creates `src-tauri/`
3. Add `wm-core` as a dependency in `src-tauri/Cargo.toml`
4. Create `src-tauri/src/commands.rs` with the core commands (initial, search, health)
5. Wrap `ApiService` methods with `invoke()` fallback (try Tauri first, fall back to HTTP for dev)
6. Set up Tauri managed state (`Arc<EngineState>`)

### Phase 2: Port All Routes

7. Port pages CRUD commands (list, get, create)
8. Port tasks board command
9. Port graph commands (stats, neighbors, full)
10. Port memory list command
11. Move `graph_rebuild_loop` to Tauri `setup()`
12. Convert SSE events to Tauri events

### Phase 3: Remove wm-server

13. Delete `apps/wm-server/` crate
14. Remove `wm-server` from workspace `Cargo.toml`
15. Remove `server` feature from `wm-cli`
16. Remove `--project` and `--port` flags from `wm-cli web` (replace with `tauri dev`)
17. Clean up any remaining `wm-server` references

### Phase 4: Polish

18. System tray icon with context menu (open, quit, about)
19. Window state persistence (position, size)
20. Native file dialogs for opening `.wm` projects
21. About dialog with version info

## Technical Notes

### Tauri Project Structure

```
apps/wm-web/
├── src-tauri/
│   ├── Cargo.toml          # depends on wm-core
│   ├── tauri.conf.json     # window config, frontendDist
│   ├── capabilities/
│   │   └── default.json    # permissions (fs, path, etc.)
│   ├── src/
│   │   ├── lib.rs          # Tauri builder, managed state
│   │   ├── commands.rs     # all API commands
│   │   └── events.rs       # event emission helpers
│   └── icons/              # app icons
├── src/
│   └── app/
│       └── services/
│           └── api.service.ts  # invoke() wrapper
└── angular.json
```

### Cargo.toml (src-tauri)

```toml
[package]
name = "wm-tauri"
version = "0.1.0"
edition = "2021"

[lib]
name = "wm_tauri_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-fs = "2"
tauri-plugin-dialog = "2"
tauri-plugin-shell = "2"
wm-core = { path = "../wm-core" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
```

### ApiService Wrapper Pattern

```typescript
import { inject, Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';

@Injectable({ providedIn: 'root' })
export class ApiService {
  private isTauri = !!(window as any).__TAURI_INTERNALS__;

  async getPage(id: string): Promise<Page> {
    if (this.isTauri) {
      return invoke<Page>('get_page', { id });
    }
    // Fallback for `ng serve` dev mode
    const res = await fetch('/api/pages/get', { method: 'POST', body: JSON.stringify({ id }) });
    return res.json();
  }
}
```

### Development Workflow

```bash
# Terminal 1: Angular dev server (hot-reload)
cd apps/wm-web && npx ng serve

# Terminal 2: Tauri dev (opens native window, connects to Angular dev server)
cd apps/wm-web && npx tauri dev

# Production build
cd apps/wm-web && npx tauri build
```

`tauri dev` automatically connects to `http://localhost:4200` (configured in `tauri.conf.json`).

### Removing wm-server

After migration, the full deletion:
- `apps/wm-server/` directory
- `wm-server` entry in workspace root `Cargo.toml`
- `server` feature and `dep:wm-server` in `wm-cli/Cargo.toml`
- `wm_server::*` references in `wm-cli/src/main.rs`
- `--port` flag in the `web` command (now `tauri dev` handles this)

## Out of Scope

- Mobile (Tauri mobile builds are possible but not targeted)
- Auto-updater (`tauri-plugin-updater` can be added later)
- Multi-window support (single window for now)

## Open Questions

- [ ] Should the existing `wm-cli web` command remain as a convenience alias for `tauri dev`?
- [ ] Should graph layout computation run synchronously on IPC or be streamed via Tauri events for progressive rendering?
