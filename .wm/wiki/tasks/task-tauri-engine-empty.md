---
title: Tauri Engine: detect_project_root() fails — graph has 0 nodes
type: task
status: todo
---

**Severity:** High

**Observed:** Tauri app's EngineState has 0 graph nodes and 0 edges (IPC get_initial returns graph_node_count: 0). The graph is stale. Search returns 0 results. All views are empty.

**Root Cause:** `detect_project_root()` in `apps/wm-web/src-tauri/src/lib.rs:9-19` walks up from `std::env::current_dir()` looking for `.wm/`. When the binary is launched from outside the project root (e.g. from shell in `target/debug/`), it doesn't find `.wm/` and falls back to current_dir, which has no wiki pages.

**Fix:** Try the binary's own location (`std::env::current_exe()`) first, walking up from there. Or add a `--root` CLI arg to override. The `wm-cli` already resolves this correctly.

**File:** `apps/wm-web/src-tauri/src/lib.rs:9-19`

**Acceptance Criteria:**
- [ ] Tauri app shows 244+ graph nodes on startup
- [ ] Search returns results
- [ ] Graph view renders the graph canvas with nodes