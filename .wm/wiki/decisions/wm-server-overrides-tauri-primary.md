---
id: wiki:decisions:wm-server-overrides-tauri-primary
title: "Decision: Override D1 — wm-server Daemon Replaces Tauri Primary"
type: decision
status: approved
tags: [decision, architecture, tauri, server, http, daemon]
---

---
id: wiki:decisions:wm-server-overrides-tauri-primary
title: "Decision: Override D1 — wm-server Daemon Replaces Tauri Primary"
type: decision
status: approved
tags: [decision, architecture, tauri, server, http, daemon]
---
id: wiki:decisions:wm-server-overrides-tauri-primary

## Context

The existing enterprise-grade convention (D1 in `conventions/enterprise-grade.md`) locked **Tauri v2 as the primary deployment** — a desktop app bundling Angular frontend via webview + Rust backend via Tauri IPC. All processes (`wm-cli mcp`, Tauri backend, CLI commands) each created their own full `EngineState` instance (~500MB each), causing:
- 3+ copies of graph, BM25 index, embedder, vector store
- No single source of truth — MCP mutations not reflected in Web UI
- 15 hand-written Tauri commands duplicating MCP tool logic
- Angular frontend coupled to Tauri IPC with no HTTP fallback

## Decision

Override D1: Replace Tauri with a **single `wm-server` daemon** that owns the one `EngineState` and exposes both the HTTP API and the embedded Angular SPA:

```
Browser (Angular)              AI Agent (OpenCode)
       │ HTTP :4090                     │ spawns
       │                                ├── wm-cli mcp (thin proxy)
       │                                │   rmcp → HTTP
       ▼                                ▼
  ┌──────────────────────────────────────────┐
  │            wm-server (:4090)             │
  │  GET /  → Angular SPA (rust-embed)       │
  │  POST /api/* → REST handlers             │
  │  GET /api/events → SSE                   │
  │  owns: single EngineState                │
  └──────────────────────────────────────────┘
```

All Clients Connect to One Server:
- **Angular UI** → pure web app, `fetch()` to `:4090` (no Tauri IPC)
- **`wm-cli mcp`** → thin HTTP proxy, translates MCP/stdio ↔ HTTP/:4090
- **CLI commands** → HTTP calls to `:4090` where applicable

## Rationale

| Factor | Tauri (before) | wm-server (after) |
|--------|---------------|-------------------|
| EngineState copies | 3+ | 1 |
| Frontend dependency | Tauri IPC only | Standard HTTP |
| MCP ↔ Web consistency | Separate processes, stale | Same EngineState, immediate |
| Debugging | Tauri dev tools + Rust | curl + browser dev tools |
| Deployment | Tauri desktop binary | Single binary + any browser |
| Portability | Windows/macOS/Linux only | Any device with a browser |

## Consequences

**Positive:**
- Single EngineState — no stale graph, no duplication
- Angular becomes a standard web app — can be served by nginx, CDN, or embedded
- MCP proxy auto-discovers tools from server via `/api/help`
- Any HTTP client can use the API (curl, scripts, other apps)
- Tauri build step (~5min) eliminated

**Negative:**
- Browser required — no native window chrome or system tray
- Users must run `wm-server` explicitly or via launcher script
- SSE events needed for real-time updates (replaces Tauri event system)
- Migration effort: Phase 1-5 across existing codebase

**Overridden:**
- D1 in `conventions/enterprise-grade.md` — "Tauri v2 primary, all-in" → replaced by `wm-server` primary

## Related

- @doc/specs/single-http-server — Full migration spec
- @wiki/core:architecture — Current architecture overview (ARCHITECTURE-SPEC.md removed)
- @doc/conventions/enterprise-grade — Overridden convention (D1)
