---
title: 'Pattern: Durable stale marker for a failed derived-state refresh'
type: pattern
id: "wiki:patterns:durable-stale-marker-for-failed-refresh"
status: reviewed
tags: [pattern, error-handling, cache, index, reliability]
relates_to:
  - {type: references, target: wiki:tasks:refresh-code-index-drops-failed-refreshes-without-retry}
---

schema_version: 1
state: |-
  ## Problem

  A watcher-triggered refresh of a derived index can fail transiently (most often a database lock while another
  writer holds the file). If the refresh path only logs the error and returns, the triggering file event is lost
  permanently. Readers then answer from an index that silently disagrees with disk, and nothing surfaces the
  divergence — the same class of failure as the 20-second watcher-test timeout and a ten-day-old `code.db`.

  ## Solution

  Treat a failed refresh as a state transition, not a log line:

  1. **Classify before retrying.** Retry only contention failures. In this codebase turso reports
     `Locking error: Failed locking file ... File is locked by another process`; `is_lock_contention` checks those
     markers. A permanent error (missing table, corrupt database, I/O) fails fast instead of burning the budget and
     delaying the report.
  2. **Retry within a bound.** Run the refresh up to three times with exponential backoff (100 ms then 400 ms, ~0.5 s
     total) on the calling thread. This absorbs transient contention without unbounded blocking.
  3. **Persist a stale marker on exhaustion.** Write the failure text to a known marker file under the state
     directory (here `.wm/state/code-index.stale`). The marker is the durable "index may disagree with disk" bit.
  4. **Make staleness checks honor the marker.** `is_stale` returns true while the marker exists, so the next
     reader's `refresh_if_stale` rebuilds even when file-count/mtime comparisons say the index is current (a
     partially written rebuild can leave hashes that look fresh).
  5. **Clear the marker only on a successful rebuild, and never let cleanup fail the rebuild.** The watcher path,
     the read-path probe, and the manual `wm index code` command all clear it after success. If removal fails, log
     and keep the marker: leaving it causes one redundant rebuild, which is safe; reporting the rebuild as failed or
     retrying a completed rebuild is not. When the database is absent there is nothing to be stale, so clear the
     marker too.
  6. **Surface the marker to callers.** Expose it through the status surface (`wm_index_status` adds
     `code_index_stale` and `code_index_refresh_error`) and the CLI, not only the log.

  Implementation: `apps/wm-core/src/engine/code_index_refresh_service.rs` (`refresh_code_index_with_retry`,
  `is_lock_contention`, `pending_refresh_failure`, `clear_marker_after_refresh`), constant
  `CODE_INDEX_STALE_FILE` in `packages/wm-constants/src/files.rs`, watcher call site in
  `apps/wm-core/src/engine/main_engine_factory.rs`.

  ## When to Use

  - Any derived index/cache refreshed from a file event or background task where the event cannot be replayed.
  - Refresh failures that can be transient (locks, concurrent writers, temporary I/O).
  - Systems where a status or diagnostics surface already exists to report index health.

  ## When Not to Use

  - Refreshes that are already idempotent and re-triggered by every read (the mtime probe alone is enough).
  - Failures that are permanent and user-actionable (a missing database); classify them and fail fast rather than
    retrying.
  - Multi-process writers where the marker file itself would race — use the database's own transaction/version
    state instead.

  ## Failure Mode This Prevents

  `refresh_code_index_for` matched `Err(e)` and called `tracing::warn!` only. A single DB-lock collision dropped the
  event; the index under-reported until something triggered a full rebuild. Tests could not distinguish "watcher
  never fired" from "watcher fired and the refresh was discarded".

  ## Related

  - @wiki/patterns/refresh-derived-state-at-write-path — the synchronous-refresh counterpart; this pattern covers
    what to do when that refresh fails.
  - @wiki/tasks/refresh-code-index-drops-failed-refreshes-without-retry
  - @wiki/specs/code-index-refresh-retry
questions:
  - id: problem_kind
    type: choice
    instructions: What kind of problem does this pattern solve?
    options:
    - architecture
    - api-design
    - data-model
    - error-handling
    - performance
    - testing
    - ui
    - tooling
    - workflow
  - id: preconditions_required
    type: noul
    instructions: This pattern requires specific preconditions to be met.
  - id: complexity
    type: score
    instructions: How complex is applying this pattern?
    levels:
    - trivial
    - simple
    - moderate
    - complex
  - id: language_specific
    type: noul
    instructions: This pattern is specific to a programming language.
answers:
  problem_kind: error-handling
  preconditions_required: true
  complexity: simple
  language_specific: false
