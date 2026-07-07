---
id: qrdfbt
title: 'P0 Rust fixes: blocking I/O in async, flush deadlock, entries.flatten, mutex poisoning'
status: done
priority: high
labels:
  - review
  - rust
  - p0
createdAt: '2026-07-07T08:46:20.408Z'
updatedAt: '2026-07-07T09:04:39.948Z'
timeSpent: 184
spec: specs/p0-rust-fixes-blocking-io-flush-deadlock-entries-flatten-mutex-poisoning
---
# P0 Rust fixes: blocking I/O in async, flush deadlock, entries.flatten, mutex poisoning

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Fix 4 P0 items from rust-reviewer:

1. **Blocking I/O in tokio tasks** (engine.rs:314-361 WriteChannel::spawn_consumer, engine.rs:719-763 audit log consumer) — Wrap std::fs::write/create_dir_all/OpenOptions/remove_file in `tokio::task::spawn_blocking()`. The audit log consumer's file operations should also be wrapped.

2. **WriteChannel::flush deadlock** (engine.rs:301-306) — Replace `std::sync::mpsc::channel` + `rx.recv()` with `tokio::sync::oneshot::channel` + `rx.await`. Make flush `pub async fn flush(&self)`. Update WriteOp::Flush enum variant to use `tokio::sync::oneshot::Sender<()>`.

3. **entries.flatten() silently drops I/O errors** (tools.rs ~1615) — Replace `for entry in entries.flatten()` with `for entry in entries { match entry { Ok(e) => ..., Err(err) => tracing::warn!(...) } }`.

4. **Mutex poisoning silently ignored** (engine.rs) — Though `if let Ok` is idiomatic Rust, add `tracing::error!` logging when Mutex locks are poisoned for wiki_dir_mtime, memory_dir_mtime, and IndexScheduler cancel_tx so the error isn't completely silent.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
P0 Rust fixes implemented:
1. WriteChannel::flush: Changed WriteOp::Flush to use tokio::sync::oneshot, flush() is now async.
2. spawn_consumer: All file I/O wrapped in spawn_blocking to avoid blocking tokio workers.
3. Audit log consumer: File operations wrapped in spawn_blocking.
4. entries.flatten() in tools.rs ~1606: Replaced with explicit match + tracing::warn! on errors.
5. Mutex poisoning: Added tracing::error! on poisoned locks for wiki_dir_mtime, memory_dir_mtime, and IndexScheduler cancel_tx.
All 108 tests pass.
<!-- SECTION:NOTES:END -->

