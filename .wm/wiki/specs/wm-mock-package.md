---
title: wm-mock — Tauri IPC Mocking Package
type: spec
tags: [spec, mock, tauri, testing, ipc]
status: draft
---

## Overview

Create `packages/wm-mock` — a lightweight mock data package for Tauri IPC calls. It reuses the existing WireMock-compatible JSON stub format and adds zero-config scenario switching. No HTTP server, no separate process — pure in-process mocking for the Tauri desktop app.

## Motivation

The old `wm-server` HTTP backend and `wm-mock-server` (Hono/WireMock) are deleted. There is no HTTP pathway — only Tauri IPC. A new package is needed that:

- Mocks `invoke('search', {...})` directly, not `POST /api/search`
- Reuses existing `mappings/*.json` stub files without changes
- Supports scenario activation (empty search, error states, etc.)
- Works in WebdriverIO (WDIO browser mode) for E2E tests and in `tauri dev` for development

## Requirements

### FR-1: Stub Format Compatibility
The existing WireMock-compatible JSON stub format must work without changes:

```json
{
  "request": { "method": "POST", "urlPath": "/api/search" },
  "response": {
    "status": 200,
    "headers": { "Content-Type": "application/json" },
    "jsonBody": { "success": true, "results": [...], "total": 5 }
  }
}
```

### FR-2: IPC Command Translation
A `CMD_MAP` translation table maps Tauri IPC command names to the HTTP URL paths in stub files:

| IPC Command | HTTP Path | Stub File |
|---|---|---|
| `search` | `/api/search` | `api-search.json` |
| `get_initial` | `/api/initial` | `api-initial.json` |
| `list_pages` | `/api/pages/list` | `api-pages-list.json` |
| `get_page` | `/api/pages/get` | `api-pages-get.json` |
| `create_page` | `/api/pages/create` | `api-pages-create.json` |
| `task_board` | `/api/tasks/board` | `api-tasks-board.json` |
| `list_memory` | `/api/memory/list` | `api-memory-list.json` |
| `get_graph_full` | `/api/graph/full` | `api-graph-full.json` |
| `get_graph_stats` | `/api/graph/stats` | `api-graph-stats.json` |
| `get_graph_neighbors` | `/api/graph/neighbors` | `api-graph-neighbors.json` |

### FR-3: Scenario Switching
Load stub files from subdirectories (`mappings/vpp/`) and activate them:

```typescript
registry.setDefaults(loadStubs('mappings'));       // base stubs
registry.activateScenario(loadStubs('mappings/vpp')); // scenario overrides
registry.reset();                                   // back to defaults
```

### FR-4: Two Adapters

| Adapter | Mechanism | For |
|---------|-----------|-----|
| `mockIPC()` | WebdriverIO `browser.mockIPC()` | WDIO E2E tests |
| `dev invoke` | Fake `invoke()` injection | `tauri dev` development |

### FR-5: Dev Mode (`tauri dev`)
When running `tauri dev`, provide a way to load mock data instead of calling real Tauri commands. A `?mock=true` query param activates the mock registry, and the Angular app injects the fake `invoke` function.

## Non-Functional Requirements

- NFR-1: Zero external HTTP dependencies (no Hono, no express, no server process)
- NFR-2: All existing `mappings/*.json` files are usable without changes
- NFR-3: The `wm-mock-server` and `wm-web-e2e/mock-server` packages are deleted (obsolete)

## Acceptance Criteria

- [ ] AC-1: Core `MockRegistry` loads stubs from JSON files and matches by method + urlPath
- [ ] AC-2: `CMD_MAP` translates all 10 Tauri IPC commands to HTTP paths
- [ ] AC-3: Scenarios can be activated and reset without page reload
- [ ] AC-4: `mockIPC()` adapter registers all commands with WebdriverIO
- [ ] AC-5: `dev invoke` adapter works when `?mock=true` is set in `tauri dev`
- [ ] AC-6: `wm-mock-server` package is deleted
- [ ] AC-7: `wm-web-e2e/mock-server` references are cleaned up
- [ ] AC-8: Package builds with zero TypeScript errors

## Package Structure

```
packages/wm-mock/
├── package.json
├── tsconfig.json
├── src/
│   ├── index.ts                  ← public API
│   ├── core/
│   │   ├── types.ts              ← StubMapping, StubRequest, StubResponse, ParsedStub
│   │   ├── matcher.ts            ← matchStub(method, pathname, params, stubs)
│   │   ├── stub-loader.ts        ← loadStubs(fileReader, dir), validateStub()
│   │   ├── registry.ts           ← MockRegistry (defaults, scenarios, dynamic, find)
│   │   └── cmd-map.ts            ← CMD_MAP translation table
│   └── adapters/
│       ├── tauri-mock.ts         ← registerTauriMocks(registry) for WDIO
│       └── dev-mock.ts           ← createMockInvoke(registry) for tauri dev
```

## Technical Notes

### Core Flow

```
invoke('search', { payload: { q: "mcp" } })
  → CMD_MAP['search']
    → { method: 'POST', urlPath: '/api/search' }
      → MockRegistry.find('POST', '/api/search', { q: "mcp" })
        → matchStub filters by method + urlPath, prefers query-param match
          → returns stub.response.jsonBody
```

### Dev Mode Integration

In `apps/wm-web/src/main.ts`, add mock mode detection:

```typescript
if (window.location.search.includes('mock=true')) {
  const { createMockInvoke, MockRegistry } = await import('@vpp-rag/mock');
  const registry = new MockRegistry();
  // Stubs are bundled or fetched
  const stubs = await loadStubs('/mocks');
  registry.setDefaults(stubs);
  (window as any).__MOCK_INVOKE__ = createMockInvoke(registry);
  (window as any).__TAURI_INTERNALS__ = {}; // force Tauri IPC path
}
```

Stub JSON files are served as static assets by Tauri's dev server, or pre-bundled via Vite.

## Deletions

When this spec is implemented:

- `packages/wm-mock-server/` — entire directory deleted (Hono/WireMock server)
- `apps/wm-web-e2e/helpers/mock-manager_helper.js` — replaced by new MockManager
- `apps/wm-web/proxy.conf.json` — deleted (no more HTTP proxy)
- `apps/wm-web/postcss.config.js` — already deleted (was replaced by .json)

## Migration Path

| Mode | Before | After |
|------|--------|-------|
| Dev | `ng serve` + proxy → mock-server :8081 | `tauri dev` + `?mock=true` → wm-mock |
| E2E | CodeceptJS + Playwright + mock-server :8081 | WDIO + `mockIPC()` + wm-mock |
| Desktop | Tauri binary + real IPC | Tauri binary + real IPC (unchanged) |

## Open Questions

- [ ] Should stub JSON files be pre-bundled (imported as JS objects) or fetched at runtime from a static directory?
- [ ] Should `wm-web-e2e` be converted to WDIO or kept as CodeceptJS with the `fetch` interceptor?
