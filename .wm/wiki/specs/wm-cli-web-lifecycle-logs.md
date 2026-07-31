---
title: wm-cli web — Lifecycle Logs + Port Propagation
type: spec
id: wiki:specs:wm-cli-web-lifecycle-logs
status: approved
tags: [approved, spec, cli, wm-server, web-ui]
---

# wm-cli web — Lifecycle Logs + Port Propagation

## Overview

`wm-cli web` currently prints only `Starting wm-server on port X...` then blocks on `child.wait()`. The user expectation: log shows `starting → started` for BOTH wm-server and wm-web. Additionally, the `--port` flag is silently ignored — wm-server hardcodes `127.0.0.1:4090`.

Verified behavior (2026-07-31):
```
2026-07-31T06:27:41.346400Z  INFO wm_cli: Starting wm-server on port 4090...
```
No further lines. `wm-cli web --port 4093` → log says 4093, curl 4093 → 000, curl 4090 → 200.

## Locked Decisions

- D1: wm-cli web logs lifecycle transitions: `Starting wm-server...` → `wm-server started` → `Starting wm-web...` → `wm-web started`
- D2: "started" = readiness confirmed (server accepting HTTP + SPA route responding), not just spawn success
- D3: wm-server accepts `--port N` and binds it; defaults to 4090 when absent
- D4: Readiness probe is a lightweight HTTP poll of the bound port with a deadline (no fixed sleep)
- D5: wm-web readiness = the SPA index route returns 200 (or spa dir is served) — same server process

## Requirements

### FR-1: wm-cli web lifecycle logs
`Commands::Web` handler logs, in order:
1. `Starting wm-server...`
2. `wm-server started` (after readiness probe succeeds on the requested port)
3. `Starting wm-web...`
4. `wm-web started` (after SPA route readiness)

If the server exits before readiness, log the failure and exit non-zero.

### FR-2: wm-server honors --port
`apps/wm-server/src/main.rs` parses `--port <n>` (default 4090) and binds it. Logs the actual bound port.

### FR-3: Readiness probe
Poll `GET http://127.0.0.1:{port}/` (or `/api/health`) with a deadline (e.g. 10s, 100ms interval). First 2xx → started.

## Acceptance Criteria

- [ ] AC-1: `wm-cli web` log contains `Starting wm-server` then `wm-server started`
- [ ] AC-2: `wm-cli web` log contains `Starting wm-web` then `wm-web started`
- [ ] AC-3: `wm-cli web --port 4999` → wm-server binds 4999 (curl 4999 → 200)
- [ ] AC-4: Lifecycle lines appear in order starting → started for both
- [ ] AC-5: Regression test covers log ordering + port propagation

## Scenarios

### Scenario 1: Normal start
**Given** a built wm-server next to wm-cli and a built SPA
**When** `wm-cli web --port 4090` is run
**Then** log shows `Starting wm-server` → `wm-server started` → `Starting wm-web` → `wm-web started`, and the UI is reachable on 4090

### Scenario 2: Custom port
**Given** `wm-cli web --port 4999`
**Then** wm-server listens on 4999, not 4090

## References

- `apps/wm-cli/src/main.rs` — Commands::Web handler (line ~1268)
- `apps/wm-server/src/main.rs` — hardcoded bind (line 46)
- `apps/wm-server/src/spa.rs` — SPA serving
- @wiki/tasks/wm-cli-web-lifecycle-logs-startingstarted-for-wm-server--wm-web-honor---port
