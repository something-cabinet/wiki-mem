---
title: Session: MCP Refactor + Tauri Migration + Sim UI + Graph
type: memory
status: draft
relates_to:
  - {type: references, target: wiki:tasks:fjadra-layout}
  - {type: references, target: wiki:tasks:webgl-labels}
  - {type: references, target: wiki:tasks:postcss-config}
  - {type: references, target: wiki:tasks:e2e-migration}
---

## Session State (2026-07-17)

Massive full-stack overhaul (~40+ commits) completed.

### Architecture
- Tauri v2 desktop app (wm-tauri) — primary frontend with 10 IPC commands
- wm-cli — CLI + MCP server for OpenCode integration
- wm-web — Angular 22 app with Sim UI, NgRx, regl/fjadra graph
- wm-server — deleted (fully replaced by Tauri IPC)

### What works
- wm-cli mcp: direct tool handlers, no HTTP proxy
- wm-tauri: builds and launches (needs WebView2)
- wm-web: builds clean with full Sim UI
- wm-mock-server: WireMock-compatible mock (IPC + HTTP + fetch)
- Graph: Canvas 2D + WebGL (regl) with force-directed layout, pan/zoom, drag, LOD

### Active tasks
- @wiki/tasks/fjadra-layout — Implement fjadra Rust force-directed layout
- @wiki/tasks/webgl-labels — Implement WebGL SDF text labels
- @wiki/tasks/postcss-config — Fix PostCSS config for Angular 22
- @wiki/tasks/e2e-migration — Migrate legacy CodeceptJS E2E to WDIO