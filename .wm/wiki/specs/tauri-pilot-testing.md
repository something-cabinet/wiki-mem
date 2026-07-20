---
title: Tauri Pilot Testing
type: spec
status: approved
tags: [spec, approved, testing, tauri, pilot]
---

## Overview

Create a comprehensive test suite for the Tauri desktop app (`apps/wm-web`) using `tauri_plugin_pilot` — the built-in JSON-RPC testing interface. Tests cover IPC command correctness, UI rendering health, and end-to-end workflows, all running against a real Tauri binary (no mocking).

## Motivation

The Tauri app currently has zero integration tests. The existing e2e tests (`apps/wm-web-e2e/` and `apps/wm-web/e2e/`) mock all Tauri IPC commands — they test the Angular UI in isolation but never verify real Rust backend behavior. `tauri_plugin_pilot` is already wired (conditional on `debug_assertions`) and exposes every registered Tauri command via JSON-RPC 2.0 over a Windows Named Pipe, plus full DOM interaction. This spec defines the test infrastructure and test suite needed to close that gap.

## Locked Decisions

- D1: Tests use `tauri_plugin_pilot`'s JSON-RPC 2.0 interface over Named Pipe (Windows) or Unix socket (macOS/Linux)
- D2: IPC command tests are the highest priority — they test real Rust backend behavior without WebView dependency
- D3: Test harness uses the `tauri-pilot` CLI binary (~/.cargo/bin/tauri-pilot) as client — no platform-specific pipe/socket code
- D4: Two-tier test data: read-only tests (get/search/graph/stats) run against real project wiki; CRUD tests (create/update/delete) use a temp wiki directory
- D5: One app launch per test group — read-only group against real wiki, CRUD group against temp wiki
- D6: Event assertions (compute_layout) via Rust debug-only commands (`get_captured_events`, `clear_captured_events`) rather than JS eval polling
- D7: Test runner is a Rust integration binary at `apps/wm-web/src-tauri/tests/pilot_runner/` as a workspace member
- D8: Auto-build with cache check: use existing binary if newer than sources, otherwise build

## Requirements

### FR-1: Test Harness
- A standalone test runner binary that builds and launches the Tauri app in debug mode
- Connects to the pilot socket/pipe by polling until ready (with timeout)
- Sends JSON-RPC 2.0 requests and validates responses
- Reports pass/fail per test case with structured output
- Shuts down the app cleanly after all tests (or on first failure)
- Cross-platform: works on Windows (Named Pipes) and Unix (Unix sockets)

### FR-2: IPC Command Coverage
- Every registered Tauri command must have at least one test:
  - `get_initial` — verify `bm25_loaded`, `graph_node_count`, `sections_indexed`
  - `list_pages` — verify returns known pages
  - `get_page` — verify correct title/type/status for a known page
  - `create_page` / `update_page` / `delete_page` — full CRUD cycle
  - `task_board` — verify board structure with expected columns
  - `search` — verify non-empty results for known query
  - `get_graph_full` / `get_graph_stats` — verify structure and counts
  - `get_graph_neighbors` — verify edge data
  - `compute_layout` — verify layout runs and emits events
- Edge cases: empty search, non-existent page ID, creating a page with minimal fields

### FR-3: UI Rendering Smoke Tests
- Navigate to each route (`/`, `/graph`, `/pages`, `/tasks`, `/memory`) and confirm the page loads
- Verify critical UI elements exist (canvas element on graph view, page list on pages view)
- Capture DOM snapshots as baselines on first run; diff against them on subsequent runs
- Test window title and basic Angular component presence

### FR-4: Workflow Tests
- **Create → Search → Graph:** Create a page with a unique title via IPC, search for it via IPC, navigate to graph view, verify node appears
- **Edit → Verify:** Create a page, update its content, fetch via IPC, confirm changes persisted
- **Layout → Render:** Open graph view, trigger `compute_layout`, wait for `graph-settled` event, verify positions applied

### FR-5: CI Integration
- A `justfile` command: `tauri-test` — launch Tauri + run pilot tests + cleanup
- Runs as a separate CI job on PRs that touch `apps/wm-web/src-tauri/`
- Must not leave orphan Tauri processes on failure (cleanup handler)

## Acceptance Criteria

- [ ] AC-1: Test harness starts Tauri app, connects via pilot, runs all IPC tests, and shuts down cleanly
- [ ] AC-2: All 15+ registered Tauri commands have at least one passing test
- [ ] AC-3: Each route in the Angular app loads without JS errors
- [ ] AC-4: Create → Verify workflow passes end-to-end
- [ ] AC-5: CI job runs tests and reports pass/fail per file
- [ ] AC-6: No orphan Tauri processes remain if tests fail or are interrupted

## Scenarios

### Scenario 1: Full IPC smoke test
**Given** the Tauri app is running with pilot enabled
**When** the test runner calls every registered command with valid parameters
**Then** each command returns a successful JSON-RPC response with expected fields

### Scenario 2: Page CRUD workflow
**Given** the Tauri app is running
**When** the test runner creates a page, retrieves it, updates its title, retrieves it again, then deletes it
**Then** each step returns the expected result and the final delete succeeds

### Scenario 3: App not found for pilot
**Given** the Tauri binary does not exist or is not built
**When** the test runner tries to launch it
**Then** the runner reports a clear error and exits non-zero without hanging

## Technical Notes

### Windows Named Pipe path
```
\\.\pipe\tauri-pilot-work.knowns.wm
```
The identifier `work.knowns.wm` comes from `tauri.conf.json` line 4.

### JSON-RPC protocol
Requests are newline-delimited JSON (one object per line). The `ipc` method calls any registered Tauri command:
```json
{"jsonrpc":"2.0","id":1,"method":"ipc","params":{"cmd":"get_initial","args":{}}}
```

### Test structure
```
apps/wm-web/src-tauri/
  tests/
    pilot_runner/
      Cargo.toml
      src/
        main.rs        — CLI args, app lifecycle, test dispatch
        harness.rs     — launch/kill app, connect to pipe, send JSON-RPC
        tests/
          mod.rs       — test registry
          ipc.rs       — IPC command tests (FR-2)
          ui.rs        — UI rendering tests (FR-3)
          workflow.rs  — end-to-end workflow tests (FR-4)
```

### Dependencies
- `serde_json` for JSON-RPC messages
- `serde` for response deserialization
- On Windows: `windows-sys` or `miow` crate for Named Pipe client
- On Unix: `tokio::net::UnixStream` or `std::os::unix::net::UnixStream`

## Out of Scope
- DOM-level assertions beyond element presence (snapshot/diff is baseline-only, not per-test assertions)
- Performance/stress testing via pilot
- Mobile platform testing (Android/iOS)
- Replacing the existing CodeceptJS/WDIO e2e tests — pilot tests complement them

## Open Questions

- None — all gray areas resolved in D1-D8.
