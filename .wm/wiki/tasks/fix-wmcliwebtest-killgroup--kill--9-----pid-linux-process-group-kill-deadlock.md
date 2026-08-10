---
title: Fix wm_cli_web_test kill_group — kill -9 -- -PID (Linux process-group kill deadlock)
type: task
tags:
- bug
- cli
- tests
- release-blocking
status: done
priority: urgent
acceptance_criteria:
- text: kill_group uses POSIX 'kill -9 -- -{pid}' form (-- terminator before negative PID)
  checked: false
- text: wm_cli_web tests terminate wm-cli+wm-server processes on Linux (no orphan processes)
  checked: false
- text: wm_cli_web_test passes on macOS and CI ubuntu-latest
  checked: false
- text: No unbounded child.wait() deadlock in test Drop/cleanup
  checked: false
relates_to:
- type: implements
  target: wiki:specs:wm-cli-web-test-kill-group-fix
---

Committed wm_cli_web_test kill_group used `Command::new("kill").args(["-9", &format!("-{}", pid)])` → `kill -9 -1234`. On macOS BSD kill this sends SIGKILL to process group 1234 (works). On Linux procps kill (getopt-based) `-1234` is parsed as option characters → kill errors and sends NOTHING, silently swallowed by `let _ =` → wm-cli + wm-server survive → test's `child.wait()` in kill_group/Drop deadlocks forever. Confirmed by CI: both wm_cli_web tests hung >60s (job timed out at 30 min), orphan cleanup showed 2 wm-cli + 2 wm-server still alive. Local macOS repro passed in 5.26s. Fix: add `--` terminator → `kill -9 -- -1234` (POSIX, documented in BSD man page as `kill -- -117`). Applied + verified: 3/3 tests pass locally (1.42s); `kill -9 -- -999999` now yields 'no such process' not 'invalid option'.