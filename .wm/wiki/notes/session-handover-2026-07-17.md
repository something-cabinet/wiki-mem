---
title: Session Handover — MCP Refactor + Tauri Migration + Sim UI + Graph
type: note
tags: [handover, session, architecture]
---

## Session Summary

This session was a massive full-stack overhaul of the WM project covering approximately 40+ commits across the entire codebase.

## Current State

### What works
- `wm-cli mcp` — MCP server with direct tool handlers (no HTTP proxy)
- `wm-cli` — CLI commands (search, page, task, etc.)
- `wm-tauri` — Tauri v2 desktop app, builds and launches (57MB debug, needs WebView2)
- `wm-web` — Angular 22 app with full Sim UI components, builds clean
- `packages/wm-mock-server` — WireMock-compatible mock with IPC + HTTP + fetch adapters
- Mock server mappings: 11 stub files covering all API endpoints
- E2E: WebdriverIO + CodeceptJS test structures, 14 journeys (need WDIO config finalization)

### Graph View
- Canvas 2D renderer: force-directed layout, pan/zoom, drag, hover tooltip, edge labels with LOD
- WebGL renderer (regl): instanced nodes, batched edges, 100k+ capable, wired with auto-fallback
- Spec: `obsidian-graph-view.md` and `webgl-graph-rendering.md`

### Architecture
- **Primary**: Tauri v2 desktop app (wm-tauri) with 10 IPC commands
- **Legacy**: wm-server deleted (fully replaced by Tauri IPC)
- **MCP**: wm-cli mcp still serves OpenCode integration
- **Features**: NgRx, regl + fjadra, CSS variable theming, Sim UI components

## Project Structure

```
vpp-rag/
├── apps/
│   ├── wm-core/       ← Rust engine library (graph, search, pages, etc.)
│   ├── wm-cli/        ← CLI binary (mcp, search, page, task commands)
│   ├── wm-web/        ← Angular 22 frontend
│   │   ├── src-tauri/ ← Tauri v2 scaffold
│   │   ├── e2e/       ← WDIO test specs
│   │   └── src/libs/  ← Sim UI components + graph renderers
│   └── wm-web-e2e/    ← CodeceptJS E2E (legacy, needs migration)
├── packages/
│   ├── wm-mock-server/ ← WireMock-compatible mock (HTTP + IPC + fetch)
│   └── wm-*/          ← Extracted crate packages
└── .wm/wiki/
    ├── specs/
    │   ├── obsidian-graph-view.md       ← Graph visualization spec
    │   ├── webgl-graph-rendering.md     ← WebGL + fjadra spec
    │   ├── tauri-desktop-migration.md   ← Tauri migration spec
    │   ├── wm-mock-package.md          ← Mock server spec
    │   └── designer-review-followup.md ← UI polish spec
    └── conventions/
        └── enterprise-grade.md         ← Architecture decisions
```

## What Needs Attention

### 1. Tauri Build
Tauri builds and launches successfully but requires WebView2 on Windows (already installed). The `wm-tauri` binary is at `target/debug/wm-tauri.exe`.

### 2. WebGL Labels
SDF text labels for edge types are implemented in Canvas 2D but not yet in WebGL. The regl renderer has placeholder colors/sizes that need real data binding.

### 3. fjadra Layout
The Rust force-directed layout (fjadra) is spec'd but not implemented. Currently using d3-force for all graphs. Add fjadra to wm-tauri's Cargo.toml and create the `start_layout` IPC command.

### 4. PostCSS Config
`postcss.config.json` (not .js!) is required for Angular 22's `@angular/build` to pick up Tailwind v4. The .js format is silently ignored.

### 5. Old E2E Tests
`apps/wm-web-e2e` (CodeceptJS) is still pointing at HTTP mock server. Need to either migrate to WDIO or update to use the mock-server's IPC adapter.

## Key Commands

```bash
# Development
cd apps/wm-web && npx ng serve              # Angular dev server (port 4200)
cd apps/wm-web && npx tauri dev             # Tauri dev with hot-reload
cargo build -p wm-tauri                     # Build Tauri binary

# Mock server
cd packages/wm-mock-server && node --experimental-strip-types src/index.ts --mappings ../../apps/wm-web-e2e/mappings --port 8081

# Testing
cd apps/wm-web && npx wdio run wdio.conf.ts # WDIO E2E tests
cd apps/wm-web-e2e && npx codeceptjs run    # Legacy E2E (CodeceptJS)

# MCP for OpenCode
cargo build -p wm-cli && wm-cli mcp         # Start MCP server
```
