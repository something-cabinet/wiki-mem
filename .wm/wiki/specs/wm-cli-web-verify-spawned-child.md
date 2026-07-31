---
title: wm-cli web — verify spawned child after readiness probe
type: spec
id: wiki:specs:wm-cli-web-verify-spawned-child
status: approved
tags: [approved, spec, cli, bug]
---

# wm-cli web — verify spawned child after readiness probe

## Overview

`run_web()` in `apps/wm-cli/src/main.rs` spawns wm-server, then `probe_until_ready(port, "/api/health")`. If a STALE wm-server already holds the port, the probe connects to the OLD process and returns 2xx — the log prints `wm-server started` (and possibly `wm-web started`) while the freshly spawned wm-server fails to bind (`Address already in use (os error 48)`) and exits 1.

Verified live (user run, eightcap-new-portal): `Starting wm-server` at 09:19:17.209, `wm-server started` at 09:19:17.240 — only 31ms elapsed, impossible for real startup → probe hit a pre-existing server. Child then exited 1.

## Locked Decisions

- D1: Probe success alone is NOT sufficient — verify the spawned child is still alive after probe success
- D2: If the child exited while the probe succeeded, another process owns the port → log a clear error (port in use) and exit non-zero
- D3: `wm-server started` is logged only when the freshly spawned process is confirmed serving
- D4: `wm-web started` must also not be claimed if the spawned server died (same guard)

## Requirements

### FR-1: Post-probe child liveness check
After `probe_until_ready(port, "/api/health")` returns a 2xx, call `child.try_wait()`:
- `Ok(Some(status))` → the spawned process died (likely EADDRINUSE) → `terminate_server`, bail with error including exit code and a hint that the port may be in use
- otherwise → proceed to log `wm-server started`

### FR-2: Guard wm-web logging
Only log `wm-web started` / the not-built note after the child liveness check passes.

## Acceptance Criteria

- [ ] AC-1: child liveness checked after probe success
- [ ] AC-2: stale-port case logs a clear error, does NOT claim started, exits non-zero
- [ ] AC-3: normal start (no stale process) unchanged — 4 lifecycle lines in order
- [ ] AC-4: wm_cli_web_test suite passes
- [ ] AC-5: cargo check --workspace + clippy clean

## References

- @wiki/tasks/wm-cli-web-false-started-when-stale-process-holds-the-port
- apps/wm-cli/src/main.rs — run_web()
- apps/wm-core/tests/wm_cli_web_test.rs — lifecycle tests
