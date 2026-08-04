---
id: wiki:patterns:learning-e2e-test-infrastructure-sync-write-fix
title: 'Learning: E2E Test Infrastructure + Sync Write Fix'
type: pattern
tags: [learning, test, e2e, write-channel, review]
---
id: wiki:patterns:learning-e2e-test-infrastructure-sync-write-fix

# Learning: E2E Test Infrastructure + Sync Write Fix

Patterns, decisions, and failures from adopting Knowns E2E test patterns, the three-review code audit, and fixing the async write race.

## Patterns

### Translating Go `t.Run()` Subtests to Rust Integration Tests
- **What:** Knowns (Go) uses `t.Run("step name", func(t *testing.T) { ... })` for subtest granularity. Rust's test framework doesn't have built-in subtests. The equivalent pattern: one test function per workflow (e.g., `test_workflow_task_lifecycle`) with step comments and descriptive assertion messages.
- **When to use:** For E2E workflows where each step builds on the previous one. A single test function with clear section comments (`// Step 1: Create page`, `// Step 2: Verify`) is cleaner than 20 separate test functions with shared mutable state.
- **Source:** This session

### Active Readiness Polling for Child Process Servers
- **What:** Instead of `thread::sleep(500ms)` to wait for a child process to start, send a lightweight probe (e.g., MCP initialize) in a loop with backoff and a hard deadline.
- **When to use:** Any test that spawns a background server process (MCP, HTTP, WebSocket). The sleep approach is flaky under CI load; polling with deadline is deterministic.
- **Source:** This session (fixed MCPClient::start)

### Cross-Platform Child Process Kill
- **What:** Use `#[cfg(windows)]` conditional compilation for process termination. Windows needs `taskkill /F /T /PID <pid>`, Unix needs `child.kill()`.
```rust
#[cfg(windows)]
{ std::process::Command::new("taskkill").args(["/F", "/T", "/PID", &id]).output(); }
#[cfg(not(windows))]
{ child.kill(); }
```
- **When to use:** Any test infrastructure that needs to forcefully terminate child processes with a timeout.
- **Source:** helpers/mod.rs `kill_process()` function

## Decisions

### TRADEOFF: Synchronous Writes over Async Channel
- **Chose:** Direct `std::fs::write()` in `create_page()` and `update_page()` instead of routing through the tokio `WriteChannel`.
- **Over:** Keeping the async channel with a `flush()` barrier.
- **Outcome:** The `flush()` approach deadlocked because blocking on `std::sync::mpsc::Receiver::recv()` from a tokio context prevents the async consumer from processing the flush. Direct writes eliminate the race entirely with simpler code (~40 lines of channel infrastructure removed).
- **Recommendation:** For single-user, single-process local tools, always prefer synchronous file I/O. Only introduce async writes when you have measurable contention or a web-server workload.

### GOOD_CALL: Removing WM_PROJECT from Test Environments
- **Chose:** `cmd.env_remove("WM_PROJECT")` in all test CLI spawns.
- **Over:** Relying on test isolation via `cmd.current_dir()` alone.
- **Outcome:** Fixed 4+ flaky tests where the CLI auto-detected the host project instead of the test temp dir. The env var was inherited from the parent process.
- **Recommendation:** Always sanitize environment variables in test helpers. The parent process's environment leaks into child processes on both Unix and Windows.

### GOOD_CALL: Active MCP Readiness Polling
- **Chose:** Retry `initialize()` in 100ms intervals with 10s deadline.
- **Over:** Fixed 500ms `thread::sleep()`.
- **Outcome:** Removes flaky startup on slow CI runners. The polling typically succeeds in <200ms on fast machines and gracefully handles slow startup.
- **Recommendation:** Always use active polling with deadline for child process startup in tests.

### BAD_CALL: WriteOp::Flush with Blocking Sync Channel
- **Chose:** Added `WriteOp::Flush { done: mpsc::Sender<()> }` to the async channel and blocked the caller on `rx.recv()`.
- **Outcome:** Deadlocked on first use because the consumer runs on a `tokio::spawn` task. Blocking a tokio worker thread prevents the consumer from processing the flush op.
- **Lesson:** Never block a tokio worker thread on synchronization with another tokio task. Use `spawn_blocking` for blocking operations, or avoid the cross-thread synchronization entirely.

## Failures

### Async WriteChannel Race
- **What went wrong:** `page::create_page()` sent a `WriteOp::Write` through a `tokio::sync::mpsc::UnboundedSender`. The consumer processed it on a `tokio::spawn` task. The file wasn't on disk when `wm_index.rebuild` scanned the directory. Tests worked around this with `thread::sleep(200ms)`.
- **Root cause:** The channel returned immediately after sending, before the file was written. No flush/barrier mechanism existed.
- **Time lost:** ~2 hours across write+test+debug (plus ~30min per future developer hitting the same race).
- **Prevention:** For single-user tools, don't use async I/O channels. Direct synchronous writes are simpler and deterministic. Only add async channels when you have measured contention.

### CLI Test Project Detection
- **What went wrong:** Multiple CLI tests failed because the CLI binary detected the host project (vpp-rag) instead of the test temp directory. The `WM_PROJECT` env var from the host shell was inherited by child processes.
- **Root cause:** `run_cli()` set `cmd.current_dir(temp_dir)` but the CLI's `detect_project_root()` also checks `WM_PROJECT` env var first.
- **Time lost:** ~1 hour of debugging
- **Prevention:** Always remove `WM_PROJECT` (and similar env-based config) from test child process environments.

## Critical Promotion

### Async Write Channel Race — USE SYNC WRITES
- **Category:** failure
- **Cost:** 2 hours to discover + fix; would cost any future developer 30min+ to debug
- **Rule:** For single-user local tools using tokio, do NOT route file writes through async channels. The return-before-flush semantic causes races with any operation that reads from disk. Use `std::fs::write()` directly.