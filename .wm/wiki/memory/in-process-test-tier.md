---
title: In-process test tier with CWD guard
type: memory
id: wiki:memory:in-process-test-tier
status: active
tags: [pattern, testing, mcp, architecture]
---

## Pattern

The in-process integration tier: `setup_in_process()` (tests/helpers/inproc.rs) builds `EngineState` + `register_all_tools` over a tempdir, then tool contracts are exercised via `registry.dispatch_async` — no daemon, no stdio, no tokens. Sub-second per test.

## Why it wins

- mcp_test.rs: 2,565 → 1,079 lines, 74 → 48 tests (45 in-process + 3 genuine stdio seam tests: handshake, tools/list, tools/call incl. isError) — 74 daemon-spawning tests caused the CI accumulation hang; in-process tier runs in 1.77s.
- security_test.rs was the model citizen that proved the pattern (in-process ToolRegistry dispatch) before the overhaul generalized it.
- Process-CWD fidelity: some tools resolve wiki-relative paths against CWD, so the harness sets CWD to the project root under a process-wide `tokio::sync::Mutex` guard held for the test lifetime (serializes CWD-sensitive tests under parallel threads).
- 2026-08-12 overhaul: 8,330 → 5,153 test lines (−38%), 123 tests green, duplicates (mcp_basic.rs, cli_run.rs, hand-rolled HTTP) deleted, kill -9 teardown → SIGTERM + deadline polling.

## Keep

- Thin seam tests stay: 2-4 stdio round-trips + CLI-binary smoke slice (exit codes/stdout) + web-surface E2E with a shared daemon per binary.
- No fixed sleeps, no env-gate tests that silently pass, behavior-contract assertions only.

