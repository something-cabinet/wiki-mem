---
title: 'Learning: Post-Build Quality Pass — Spec Alignment, TUI, MCP Integration'
page_type: pattern
id: concepts/learning-post-build-quality-pass-spec-alignment-tui-mcp-integration
tags:
  - learning
  - post-build
  - tui
  - mcp
  - test
---

## Patterns

### MCP Bridge Pattern for Web UIs
- **What:** SvelteKit app spawns `wm serve` as a child process via Node's `child_process.spawn()`, communicates via JSON-RPC 2.0 over stdin/stdout. The bridge (`wm-bridge.ts`) handles `sendRequest()` → write JSON to stdin → read JSON from stdout. SvelteKit API routes (`+server.ts`) expose REST endpoints that delegate to the bridge.
- **When to use:** Any app that needs a web UI on top of an MCP-based Rust backend. Avoids adding an HTTP server to the Rust code (which would require Warp/Axum/etc.) by reusing the existing MCP protocol.
- **Limitations:** stdin/stdout piping is fragile in PowerShell test scripts. For production, use proper child process management (OpenCode does this correctly).
- **Source:** @wiki/tasks/umpd47, @wiki/tasks/s2ff4x

### create_engine() Factory Pattern for CLI/TUI
- **What:** Single `create_engine()` function in `main.rs` that creates a `VppEngine`, loads config, and rebuilds the graph. Both CLI commands and TUI mode share this one factory. When TUI mode is added later, it gets a persistent engine instead of a per-command engine.
- **When to use:** Any CLI app that needs both one-shot commands and an interactive mode. The factory becomes the single point of truth for engine initialization.

### Shared Function Extraction (lint_fix Example)
- **What:** Move business logic out of MCP tool handlers (`tools.rs`) into the core module (`graph.rs`) so both MCP tools and CLI commands can call it. `graph::lint_fix()` is called by both `wm_lint.fix` MCP tool and `wm lint fix` CLI command.
- **When to use:** When CLI commands start duplicating MCP handler logic (like lint.fix, validate, search). Extract to a shared function, call from both paths.
- **Source:** @wiki/tasks/7d3uvn

---

## Decisions

### CLI Is the Product (TUI-First) — GOOD_CALL (overrides previous)
- **Chose:** Ratatui TUI as the primary human interface. CLI auto-detects terminal → shows TUI. `--json` for scripting. `wm` without args → TUI.
- **Over:** CLI as "bootstrap only" (the previous architecture decision).
- **Tag:** GOOD_CALL (overrides)
- **Outcome:** The CLI went from a testing harness to a real product. Auto-detection works: `wm` opens TUI, `wm search query --json` works piped. No breaking changes to existing scripts.
- **Recommendation:** Keep forcing TUI refinement. The dashboard, search, and task views are scaffolded but need scroll support, paste support, and graph center selection.

### vectors.bin over SQLite — GOOD_CALL (confirmed)
- **Chose:** Flat binary format with magic bytes (`WMV\0`), version, atomic write via temp+rename.
- **Over:** `rusqlite` bundled (which was the previous recommendation in critical-patterns before being superseded).
- **Tag:** GOOD_CALL (confirmed)
- **Outcome:** Vectors.bin is 200 lines, zero deps, atomic write prevents corruption. The SQLite decision was already superseded before this session; this session confirmed the choice was correct through testing.
- **Recommendation:** The atomic write pattern (write to `.tmp`, rename) should be used everywhere files are written.

### SvelteKit over Dioxus for Web UI — GOOD_CALL
- **Chose:** SvelteKit + TypeScript + vis-network for the web UI, bridged via `wm-bridge.ts` spawning `wm serve`.
- **Over:** Dioxus (Rust-native web framework with WASM).
- **Tag:** GOOD_CALL
- **Outcome:** The SvelteKit app builds in 147ms (SSR) + 2.5s (client). vis-network provides force-directed graph layout out of the box. The Node.js bridge is ~50 lines. Dioxus would require WASM compilation, immature graph viz, and a separate web server.
- **Recommendation:** Keep the bridge pattern. If desktop native is needed later, wrap the SvelteKit app with Tauri (no Node.js at runtime).

### Giving Up on ort API Migration — SURPRISE
- **Chose:** Stopped fixing `ort 2.0.0-rc.12` API drift after fixing ~10 of 13 errors. The remaining errors were `OwnedTensorArrayData` trait not implemented for `ndarray::Array<i64, IxDyn>`.
- **Over:** Spending 2+ more hours fighting a pre-release dependency.
- **Tag:** SURPRISE
- **Outcome:** The default build works. `--features embed` works. The last few errors turned out to be a type mismatch that could have been fixed by using `ort::value::TensorRef::from_array_view` instead of `Tensor::from_array`. Dropping the approach and switching to `(Vec<i64>, Vec<i64>)` tuples fixed it.
- **Recommendation:** When facing pre-release dependency API drift (ort rc.12), try the simplest data format first (shape tuple + Vec) before fighting complex trait implementations (ndarray + OwnedTensorArrayData).

---

## Failures

### main.rs Brace Mismatch — 15min wasted
- **What went wrong:** An edit to `graph.neighbors --query` handler in `main.rs` introduced an extra closing brace that cascaded through the entire match statement. The compiler error pointed to line 1842 (file end) instead of the actual location.
- **Root cause:** The `Commands::Graph { action } => match action {` pattern means the match IS the body — there's no outer block. Adding the `if let Some(ref q) = query { }` inside the `for` loop added extra nesting that wasn't balanced.
- **Time lost:** 15 minutes
- **Prevention:** When refactoring `match action {` style arms (where the match IS the arm body), count braces carefully. Use `cargo fmt` early and often — it catches brace mismatches with clearer errors.

### onnx.rs Embed Feature API Drift
- **What went wrong:** The `--features embed` build had 13 compilation errors from `ort 2.0.0-rc.12` API changes. The session spent ~2 hours iterating through: `ort::Session` → `ort::session::Session`, `ort::Environment` → no `builder()` method (use `ort::init()` instead), `ArrayView` → `Array` for OwnedTensorArrayData, `try_extract` → `try_extract_array`.
- **Root cause:** The onnx.rs code was written against a different ort API version. `ort 2.0.0-rc.12` is a pre-release with breaking changes between versions.
- **Time lost:** ~2 hours across multiple sessions
- **Prevention:** (1) Pin ort to a specific version. (2) Build with `--features embed` as part of CI. (3) When writing against a pre-release crate, check its actual `src/` files, not its docs — the API docs lag behind.

### test-mcp-protocol.ps1 — Pipe-Based MCP Testing Doesn't Work
- **What went wrong:** The test script tried to pipe JSON-RPC requests to `wm serve`'s stdin using `& $wm serve | Select-Object`. This doesn't work because PowerShell closes stdin after the first write, and the MCP server processes each line as it arrives — but the script's piping mechanism can't maintain an open bidirectional channel.
- **Root cause:** MCP requires an open bidirectional stdin/stdout channel. Shell piping is unidirectional (you can pipe IN or pipe OUT, but not both simultaneously).
- **Time lost:** ~30 minutes
- **Prevention:** MCP tests must spawn the server as a child process with explicit stdin/stdout pipe handles. The Rust `std::process::Command` with `.stdin(Stdio::piped()).stdout(Stdio::piped())` is the correct approach. Follow Knowns' own `e2e_mcp_test.go` pattern.
