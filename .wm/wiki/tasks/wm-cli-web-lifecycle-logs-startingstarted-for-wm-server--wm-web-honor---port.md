---
title: "wm-cli web: lifecycle logs (starting→started) for wm-server + wm-web, honor --port"
id: wm-cli-web-lifecycle-logs-startingstarted-for-wm-server--wm-web-honor---port
type: task
status: done
priority: high
tags: [bug, cli, wm-server, web-ui]
acceptance_criteria:
  - text: "wm-cli web logs 'Starting wm-server' then 'wm-server started' after readiness"
  - text: "wm-cli web logs 'Starting wm-web' then 'wm-web started' when the SPA is served"
  - text: "wm-server binds the --port passed by wm-cli (no silent 4090 fallback when --port given)"
  - text: "Lifecycle lines appear in the log in order: starting → started for both wm-server and wm-web"
  - text: "Regression test covers the lifecycle log ordering + port propagation"
relates_to:
  - {type: implements, target: wiki:specs:wm-cli-web-lifecycle-logs}
---

`wm-cli web` only logs "Starting wm-server on port X..." then blocks on child.wait(). No "wm-server started" confirmation, no "Starting wm-web"/"wm-web started" lines for the SPA, and the --port arg is ignored by wm-server (hardcoded 127.0.0.1:4090 — log says 4093 but server binds 4090). Verified by running: log shows one line; curl 4093 → 000, 4090 → 200.

## Implementation Notes

Verified live after fix: `wm-cli web --port 4599` logs the 4 lifecycle lines in order (`Starting wm-server on port 4599...` → `wm-server started` → `Starting wm-web` → `wm-web started`); curl 4599 → 200, curl 4090 → 000.

Changes:
- apps/wm-cli/src/main.rs: Commands::Web → run_web() with readiness probe (raw TcpStream GET, 100ms interval, 10s deadline via READY_DEADLINE_SECS/PROBE_INTERVAL_MS consts); spawn/readiness failure exits non-zero; magic 4090 → DEFAULT_PORT (wm-constants).
- apps/wm-server/src/main.rs: port_from_args() parses --port (default wm_constants::DEFAULT_PORT); binds parsed port; listening log shows actual bound port.
- apps/wm-core/tests/wm_cli_web_test.rs (NEW): wm_cli_web_lifecycle_logs_in_order + wm_cli_web_honors_port_flag. RED confirmed first (server bound hardcoded 4090, no lifecycle lines).
- SPA built vs not-built handled: GET / 2xx → "wm-web started"; 404 → note "Web UI not built... serving API only" + still logs wm-web started (server confirmed up).
