---
title: wm_cli_web_test kill_group — POSIX `--` terminator
type: spec
id: wiki:specs:wm-cli-web-test-kill-group-fix
status: approved
tags: [approved, spec, tests, cli]
---

# wm_cli_web_test kill_group — POSIX `--` terminator

## Overview

`wm_cli_web_test.rs::kill_group()` sends `kill -9 -{pid}` to terminate the wm-cli/wm-server process group. On macOS (BSD kill) a negative PID operand signals the process group — works. On Linux (procps kill, getopt-based) `-{pid}` is parsed as option characters, kill errors out and sends NOTHING (silently swallowed by `let _ =`), so the spawned processes survive and the test's `child.wait()` in `kill_group`/`Drop` deadlocks forever.

CI evidence: both wm_cli_web tests hung >60s (job killed at 30-min timeout); runner orphan cleanup found 2 wm-cli + 2 wm-server still alive. Local macOS repro passed in 5.26s — confirming platform-dependent behavior.

## Locked Decisions

- D1: Use POSIX `--` terminator: `kill -9 -- -{pid}` (documented in BSD man page as `kill -- -117`)
- D2: Keep `process_group(0)` on spawn (unix) — the child must be a process group leader for negative-PID kill to target the group
- D3: No unbounded `child.wait()` — kill failure must not deadlock the test binary (verify kill exit; if kill fails, use a bounded wait)

## Requirements

### FR-1: Fix the kill invocation
`kill_group()` non-windows arm: `Command::new("kill").args(["-9", "--", &format!("-{}", self.child.id())])`

### FR-2: Verify no orphans
Test teardown must guarantee wm-cli + wm-server are terminated on both platforms. If the kill command reports failure, fall back to a bounded termination (e.g. `child.kill()` + bounded `wait()`), never an unbounded wait.

## Acceptance Criteria

- [ ] AC-1: kill_group uses `kill -9 -- -{pid}`
- [ ] AC-2: wm_cli_web tests terminate spawned processes on Linux CI (no orphans in runner cleanup)
- [ ] AC-3: wm_cli_web_test passes on macOS and ubuntu-latest
- [ ] AC-4: No unbounded child.wait() deadlock in test Drop/cleanup

## References

- @wiki/tasks/fix-wmcliwebtest-killgroup--kill--9-----pid-linux-process-group-kill-deadlock
- Commit 93449f9 — original kill_group (no `--`)
- macOS `man kill`: "Terminate the process group with PGID 117: `kill -- -117`"
