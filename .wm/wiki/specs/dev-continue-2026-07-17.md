---
title: Dev Continue — Post-Build Quality Pass + Angular Build Verification
type: spec
status: draft
---

## Dev Continue Spec — 2026-07-17

### Context
Post-handover from massive full-stack overhaul (~40 commits). Project is in a functional-but-needs-polish state.

### Completed
- wm init project setup
- wm upgrade → PATH + bin install
- mcpmon integration for hot-reload
- opencode.json cleanup (no relative paths)
- PostCSS config ✅ (already in .json format)
- Angular build verification ✅ (builds clean, bundle size warning only)
- Tauri build verification ✅ (compiles, 13 clippy warnings in wm-core)

### Current State
- **wm-cli**: builds, MCP server works via PATH-registered `wm`
- **wm-tauri**: builds (59MB debug), 10 IPC commands, needs WebView2 (already installed)
- **wm-web (Angular 22)**: builds clean (704kB initial, regl ESM warning)
- **Graph renderer**: Canvas 2D has full features (node colors by type, sizing by degree, edge LOD labels). WebGL regl renderer has placeholder-only buffers (all nodes same size/color, no labels)
- **E2E**: CodeceptJS legacy (7 journeys, working mock server) + WDIO scaffolded (single graph.test.ts)
- **PostCSS config**: Already `.json` format with `@tailwindcss/postcss` — ✅ done

### Next Actions
1. ~~Build verification~~ ✅
2. **Fix WebGL renderer data binding** — real node colors (from page_type), radii (from degree), edge colors (from edge_type)
3. **Add edge-type labels in WebGL** — HTML overlay approach positioned via WebGL camera transform
4. **SDF text labels** — stretch goal for native regl SDF text rendering
5. **Clippy warnings cleanup** — 13 warnings in wm-core
6. **E2E migration** — migrate CodeceptJS journeys to WDIO or wire IPC adapter

### Success Criteria
- ✅ Angular `ng build` passes clean
- ✅ wm-tauri compiles
- WM MCP server starts via mcpmon
- WebGL graph renderer shows real node colors, sizes, and edge labels