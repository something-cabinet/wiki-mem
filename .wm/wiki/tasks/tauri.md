---
id: wiki:tasks:tauri
title: "Tauri Engine: detect_project_root() fails — graph has 0 nodes"
type: task
status: todo
spec: specs/graph-bugs-review-fixes
relates_to:
  - {type: implements, target: wiki:specs:graph-bugs-review-fixes}
acceptance_criteria:
  - text: "detect_project_root() finds the project root when the binary is launched outside it (via current_exe() walk-up or a --root override)"
  - text: "The Tauri app shows graph nodes on startup (244+ nodes), search returns results, and the graph view renders the canvas"
---
id: wiki:tasks:tauri

**Severity:** High

**Observed:** Tauri app's EngineState has 0 graph nodes and 0 edges (IPC get_initial returns graph_node_count: 0). The graph is stale. Search returns 0 results. All views are empty.

**Root Cause:** `detect_project_root()` in `apps/wm-web/src-tauri/src/lib.rs:9-19` walks up from `std::env::current_dir()` looking for `.wm/`. When the binary is launched from outside the project root (e.g. from shell in `target/debug/`), it doesn't find `.wm/` and falls back to current_dir, which has no wiki pages.

**Fix:** Try the binary's own location (`std::env::current_exe()`) first, walking up from there. Or add a `--root` CLI arg to override. The `wm-cli` already resolves this correctly.

**File:** `apps/wm-web/src-tauri/src/lib.rs:9-19`

**Acceptance Criteria:**
- [ ] Tauri app shows 244+ graph nodes on startup
- [ ] Search returns results
- [ ] Graph view renders the graph canvas with nodes