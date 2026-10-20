---
title: 'turso 0.7 locking facts: open lock, busy_timeout scope, deferred rollback'
type: memory
tags: [concurrency, turso, lock, transaction, code-intel]
status: active
relates_to:
  - {type: references, target: wiki:decisions:code-index-db-serialization}
---

Verified against turso 0.7.0 source and two-process probes: (1) a non-blocking exclusive fcntl lock is taken at Builder::build(), not per transaction, and held until Database drops; (2) Connection::busy_timeout (or PRAGMA busy_timeout) covers statement-level Busy only — the open-time LockingError bypasses it, so retrying open is the caller's job; (3) BEGIN IMMEDIATE works (EXCLUSIVE == IMMEDIATE, WAL is the only journal mode); (4) Transaction::drop defaults to Rollback but defers it to the next connection access — use an explicit helper; (5) multiple connections in one process are safe (one Database per file), multiple processes are rejected by default; (6) experimental multiprocess WAL is unsafe with mixed-mode opens — issue #9362 reports silent write loss. Full reference: @wiki/decisions/code-index-db-serialization