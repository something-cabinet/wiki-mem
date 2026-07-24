---
title: P0 Rust Fixes — Blocking I/O, Flush Deadlock, Entries Flatten, Mutex Poisoning
type: spec
tags:
  - spec
  - approved
  - rust
  - p0
---
id: wiki:specs:p0-rust-fixes-blocking-io-flush-deadlock-entries-flatten-mutex-poisoning

## Overview

Fix 4 P0 (critical) Rust bugs identified by rust-reviewer. These are correctness and reliability issues that can cause hangs, silent data loss, or crashes in production use.

## Locked Decisions

Decisions were made during the rust-reviewer code review and are carried forward:

- D1: Blocking I/O in tokio tasks must be wrapped in `spawn_blocking` — no exceptions
- D2: WriteChannel::flush must use async oneshot channel, not sync mpsc
- D3: `entries.flatten()` must be replaced with explicit error handling and logging
- D4: Mutex poisoning must be logged via `tracing::error!` to avoid silent failures

## Requirements

### Functional Requirements

- FR-1: All blocking filesystem I/O in tokio async tasks must use `spawn_blocking`
- FR-2: WriteChannel::flush must not block the tokio worker thread
- FR-3: Directory read errors in tools.rs must be logged, not silently dropped
- FR-4: Mutex lock poisoning must produce visible error output

### Non-Functional Requirements

- NFR-1: Zero new deadlock or hang scenarios
- NFR-2: All fixes must compile with `cargo build` without new warnings

## Acceptance Criteria

- [ ] AC-1: `std::fs::write`/`create_dir_all`/`OpenOptions`/`remove_file` calls in WriteChannel::spawn_consumer are wrapped in `spawn_blocking`
- [ ] AC-2: Audit log consumer file operations in engine.rs:719-763 are wrapped in `spawn_blocking`
- [ ] AC-3: WriteChannel::flush uses `tokio::sync::oneshot` channel with `async fn flush(&self)` — no `rx.recv()` on worker thread
- [ ] AC-4: WriteOp::Flush carries a `tokio::sync::oneshot::Sender<()>` instead of sync sender
- [ ] AC-5: `entries.flatten()` in tools.rs ~1615 replaced with explicit `for entry in entries { match entry { ... } }` with `tracing::warn!` on errors
- [ ] AC-6: `tracing::error!` added when `wiki_dir_mtime`, `memory_dir_mtime`, or `IndexScheduler cancel_tx` locks are poisoned
- [ ] AC-7: `cargo build` and `cargo test` pass without new warnings

## Scenarios

### Scenario 1: WriteChannel::flush during concurrent writes
**Given** multiple concurrent page writes via the tokio runtime
**When** a caller invokes `flush()` to synchronize
**Then** flush must return only after all pending writes complete, without blocking the tokio worker thread

### Scenario 2: Directory read with corrupt entries
**Given** a wiki directory with an unreadable file (permissions, encoding, etc.)
**When** `entries` iterator encounters the error while scanning
**Then** the error must be logged via `tracing::warn!` and iteration continues to remaining entries

### Scenario 3: Mutex panic recovery
**Given** a panic occurs inside a mutex-protected section in engine.rs
**When** another task attempts to acquire the poisoned mutex
**Then** `tracing::error!` must log the context (which mutex, which operation)

## Technical Notes

- File locations from review: engine.rs lines 301-306 (flush), 314-361 (spawn_consumer), 719-763 (audit log), tools.rs ~1615 (entries.flatten)
- The WriteChannel is a `std::sync::mpsc::channel` currently; flush uses `rx.recv()` which blocks the tokio worker — root cause of the deadlock
- All 4 items were classified P0 by rust-reviewer during the cross-entity search review pass
