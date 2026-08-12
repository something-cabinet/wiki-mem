---
title: 'Refactor: test suite — in-process tier, kill daemon-spawning tests'
type: task
id: wiki:tasks:test-suite-simplification
status: done
acceptance_criteria:
- "Library-level integration tier (EngineState + register_all_tools + dispatch_async, security_test pattern) absorbs ~70 of the 74 mcp_test.rs tests"
- "Duplicate helpers deleted: mcp_basic.rs, cli_run.rs, hand-rolled HTTP (http_post/http_status) in tests"
- "No per-test daemon/process spawns remain; one shared daemon per binary where a daemon is needed"
- "semantic_test.rs env-gate removed or moved to an optional job"
- "nextest with per-test timeout configured; CI matrix reduced to 3 jobs; leftover 'remove after diagnosis' CI job deleted"
- "Test line count shrinks ~40% from 9,576 without losing behavior coverage"
- "All tests pass on CI and locally; zero warnings"
---

## Finding

Oracle test review (2026-08-12): 9,576 lines of "testing the plumbing" — 74 tests each spawning a daemon, kill -9 process-group teardown, counterfeit server.json, 4 hand-rolled HTTP implementations, assertions on log prose, env-gated test that cannot fail, #[ignore]d benchmarks, zero real Angular tests, CI shaped around self-inflicted wounds. The correct pattern (security_test.rs, in-process registry dispatch) already exists in-tree.

## Files

- apps/wm-core/tests/mcp_test.rs (2,565 → ~600 lines)
- apps/wm-core/tests/helpers/mcp_basic.rs, cli_run.rs (delete), mcp.rs, cli.rs, http_daemon.rs (dedupe)
- apps/wm-core/tests/wm_cli_web_test.rs (1,025 → ~300; keep cross-token/singleton contracts)
- apps/wm-core/tests/semantic_test.rs (env-gate), stress_test.rs (ignore/d or criterion)
- apps/wm-core/tests/file_watcher_test.rs (make it test the actual watcher)
- .github/workflows/ci.yml (matrix cleanup; delete suspect-test job)

## Severity

Medium — test quality; CI hang failure class dies structurally.

## Related

- @wiki/specs/test-suite-simplification
- @wiki/tasks/cli-mcp-in-process-refactor (Phase 1 — must land first)
- @wiki/concepts/ci-mcp-test-hang-daemon-accumulation

