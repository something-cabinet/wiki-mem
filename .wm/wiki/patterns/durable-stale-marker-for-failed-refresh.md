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

  A watcher-triggered refresh of a derived index can fail transiently (for example a SQLite/turso DB lock while
  another writer holds the file). If the refresh path only logs the error and returns, the triggering file event is
  lost permanently. Readers then answer from an index that silently disagrees with disk, and nothing surfaces the
  divergence — the same class of failure as the 20-second watcher-test timeout and a ten-day-old `code.db`.

  ## Solution

  Treat a failed refresh as a state transition, not a log line:

  1. **Retry within a bound.** Run the refresh up to three times with exponential backoff (100 ms then 400 ms, ~0.5 s
     total) on the calling thread. This absorbs transient contention without unbounded blocking.
  2. **Persist a stale marker on exhaustion.** Write the failure text to a known marker file under the state
     directory (here `.wm/state/code-index.stale`). The marker is the durable "index may disagree with disk" bit.
  3. **Make staleness checks honor the marker.** `is_stale` returns true while the marker exists, so the next
     reader's `refresh_if_stale` rebuilds even when file-count/mtime comparisons say the index is current (a
     partially written rebuild can leave hashes that look fresh).
  4. **Clear the marker only on a successful rebuild.** The watcher path, the read-path probe, and the manual
     `wm index code` command all clear it after success.
  5. **Surface the marker to callers.** Expose it through the status surface (`wm_index_status` adds
     `code_index_stale` and `code_index_refresh_error`) and the CLI, not only the log.

  Implementation: `apps/wm-core/src/engine/code_index_refresh_service.rs` (`refresh_code_index_with_retry`,
  `pending_refresh_failure`, `clear_refresh_failure`), constant `CODE_INDEX_STALE_FILE` in
  `packages/wm-constants/src/files.rs`, watcher call site in `apps/wm-core/src/engine/main_engine_factory.rs`.

  ## When to Use

  - Any derived index/cache refreshed from a file event or background task where the event cannot be replayed.
  - Refresh failures that can be transient (locks, concurrent writers, temporary I/O).
  - Systems where a status or diagnostics surface already exists to report index health.

  ## When Not to Use

  - Refreshes that are already idempotent and re-triggered by every read (the mtime probe alone is enough).
  - Failures that are permanent and user-actionable (a missing database); a retry loop only delays the error.
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