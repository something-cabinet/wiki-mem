---
title: wm-cli web — Review Fixes (Magic Values, Const Duplication, Log Honesty)
type: spec
id: wiki:specs:wm-cli-web-review-fixes
status: approved
tags: [approved, spec, cli, refactor, no-magic-values]
---

# wm-cli web — Review Fixes (Magic Values, Const Duplication, Log Honesty)

## Overview

Review of commit `93449f9` (wm-cli web lifecycle + --port) against project rules surfaced 4 rule violations and 1 logic finding. All are in code committed for v0.3.7. Fix locally; do NOT push (v0.3.7 release CI is running on the pushed tag).

## Findings

| # | Severity | Rule | Finding |
|---|----------|------|---------|
| V1 | High | no-magic-values | `[0u8; 4096]` buffer in `http_status` (apps/wm-cli/src/main.rs:919) and test (apps/wm-core/tests/wm_cli_web_test.rs:60,81) |
| V2 | High | no-magic-values | `"127.0.0.1"` repeated literal (main.rs:909,915; apps/wm-server/src/main.rs; test:50,56) |
| V3 | Medium | no-magic-values | `Duration::from_secs(1)` read timeout (main.rs:913; test:54) |
| V4 | Medium | shared-code conventions | `READY_DEADLINE_SECS` duplicated with divergent values: prod `10` (main.rs), test `30` (wm_cli_web_test.rs) |
| L1 | Medium | spec conformance | `probe_until_ready(port, "/")` `None` arm logs `wm-web started` without confirmation; non-2xx also logs it after a note. AC-2: "started when the SPA is served" |

## Locked Decisions

- D1: Buffer size, host, and read-timeout become named consts in `wm-constants` (shared — used by wm-cli, wm-server, wm-core tests) or local consts where single-use
- D2: `READY_DEADLINE_SECS` moves to `wm-constants` (single source, one value). Tests use the shared const so they verify prod behavior
- D3: `wm-web started` logged ONLY on 2xx from GET /. Timeout → log "web UI not confirmed" style note; non-2xx → existing note path, no "started" claim
- D4: No push — keep changes in working tree

## Requirements

### FR-1: Named constants (V1–V3)
- `[0u8; 4096]` → const (e.g. `HTTP_PROBE_BUF_LEN` in wm-constants, or local `const` if single-use)
- `"127.0.0.1"` → shared const (e.g. `LOCALHOST_ADDR` in wm-constants) used by wm-cli, wm-server, tests
- `from_secs(1)` read timeout → named const

### FR-2: Single-source deadline (V4)
- `READY_DEADLINE_SECS` in wm-constants (value 10). wm-cli and tests both import it. Tests no longer override to 30.

### FR-3: Honest wm-web started log (L1)
- GET / 2xx → `wm-web started`
- timeout → note (e.g. "web UI not confirmed (GET / no response); API only") WITHOUT claiming started
- non-2xx → existing note path (e.g. "Web UI not built (GET / returned {code}); wm-server serving API only") WITHOUT claiming started — or keep the informational "wm-web started" only when server is confirmed? Decision D3: no started claim on non-2xx/timeout.

## Acceptance Criteria

- [ ] AC-1: No magic 4096 / 127.0.0.1 / 1s literals in wm-cli, wm-server, or wm_cli_web_test
- [ ] AC-2: READY_DEADLINE_SECS single source in wm-constants, one value, both consumers import it
- [ ] AC-3: wm-web started only on 2xx from GET /
- [ ] AC-4: cargo check --workspace + clippy clean; wm_cli_web_test passes
- [ ] AC-5: No push — changes stay in working tree

## References

- @wiki/tasks/wm-cli-web-review-fixes
- Commit 93449f9 — original implementation
- `packages/wm-constants/src/defaults.rs` — existing DEFAULT_PORT const
