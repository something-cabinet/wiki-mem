---
title: 'wm-cli web: false ''started'' when stale process holds the port'
id: wiki:tasks:wm-cli-web-false-started-when-stale-process-holds-the-port
type: task
status: done
priority: high
tags: [bug, cli, wm-server, release-followup]
acceptance_criteria:
  - text: "run_web verifies the spawned child is still alive after readiness probe succeeds"
  - text: "If child exited while probe succeeded (stale process owns the port), log 'Address already in use'-style error and exit non-zero — do NOT log 'wm-server started'"
  - text: "'wm-server started' is only logged when the freshly spawned process is confirmed serving"
  - text: "Existing wm_cli_web_test suite still passes"
  - text: "cargo check --workspace + clippy clean"
relates_to:
  - {type: implements, target: wiki:specs:wm-cli-web-verify-spawned-child}
---

In run_web (apps/wm-cli/src/main.rs), the readiness probe can succeed against a STALE wm-server already bound to the port. The newly spawned wm-server then fails with 'Address already in use (os error 48)' and exits 1, but the log already printed 'wm-server started' + 'wm-web started' (false positives). Verified live: 31ms between 'Starting wm-server' and 'wm-server started' — probe hit the old process. Fix: after probe succeeds, check child.try_wait() — if the child exited, bail with a clear 'port in use by another process' error and never claim started.