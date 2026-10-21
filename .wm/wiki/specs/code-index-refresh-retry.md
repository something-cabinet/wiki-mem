---
title: Code Index Refresh Retry — bounded retry + durable stale marker
type: spec
id: "wiki:specs:code-index-refresh-retry"
status: approved
tags: [code-intel, reliability, observability, spec]
acceptance_criteria:
  - checked: false
    text: Spec reviewed and its requirements are implemented
---

schema_version: 1
state: |-
  ## Goal

  Stop `refresh_code_index_for` from dropping a failed watcher-triggered refresh silently. A lock-contention failure
  must be retried within a bound, and when the bound is exhausted the index must be marked stale durably so the next
  reader rebuilds, with the failure visible to callers instead of a single log line.

  ## Verified current state

  - `apps/wm-core/src/engine/main_engine_factory.rs:141` — `refresh_code_index_for` matched `Err(e)` and only called
    `tracing::warn!`. No retry, no state change, no return value.
  - `apps/wm-core/src/engine/main_engine_factory.rs:334` — the watcher thread calls it once per debounced batch.
  - `apps/wm-core/src/engine/code_index_refresh_service.rs:15,37` — `refresh_code_index` / `refresh_if_stale` return
    `Result` and detect staleness only by comparing on-disk file count/mtime against stored hashes.
  - `packages/wm-tools/src/tools/index.rs:200` — `wm_index_status` is the caller-facing surface for code-index
    state (`code_index_age_seconds`).

  Lock collisions are the dominant transient failure: turso reports
  `Locking error: Failed locking file '.../code.db-wal'. File is locked by another process`
  (@wiki/tasks/wiki-tool-reliability-wm-index-code-cannot-run-while-an-mcp-server-is-open-codedb-lock).

  ## Functional requirements

  - **FR-1** Classify the failure before retrying: only lock-contention errors are retried, with bounded exponential
    backoff. A permanent error (missing table, corrupt DB, I/O) fails fast instead of burning the budget.
  - **FR-2** When the retry budget is exhausted, a durable stale marker is written under `.wm/state/` so
    `refresh_if_stale` rebuilds on the next read even if file metadata comparisons say the index is current.
  - **FR-3** A successful refresh (watcher, `refresh_if_stale`, or `wm index code`) removes the marker. A failure to
    remove the marker is logged and leaves the marker in place (fail-safe); it must not turn a completed rebuild into
    a failed refresh or consume retry attempts. When no `code.db` exists there is no code index to be stale, so the
    marker is cleared.
  - **FR-4** The marker is exposed through `wm_index_status` (and the CLI code-index output), not only logs.

  ## Non-functional requirements

  - **NFR-1** Bounded blocking: total retry sleep stays under one second on the watcher thread.
  - **NFR-2** Existing JSON shapes stay additive-only.
  - **NFR-3** Project rules apply: no comments, no `else`, no `#[allow(...)]`, named constants, functions at most
    20 lines, TDD with the failing test written first.

  ## Design

  - Add `CODE_INDEX_STALE_FILE` to `packages/wm-constants/src/files.rs`.
  - In `code_index_refresh_service`:
    - `retry_refresh` checks `is_lock_contention` against `LOCK_CONTENTION_MARKERS` (`Locking error`,
      `locked by another process`) before scheduling another attempt; non-lock errors return immediately.
    - `refresh_code_index_with_retry(project_root)` runs the refresh up to `REFRESH_ATTEMPTS` times with
      `REFRESH_BACKOFF * 4^attempt` sleeps, and on exhaustion writes the error text to the marker file.
    - `is_stale` returns true while the marker exists, so `refresh_if_stale` rebuilds.
    - `refresh_code_index` and `refresh_if_stale` clear the marker after a successful rebuild via
      `clear_marker_after_refresh`, which logs and keeps the marker if removal fails; both also clear it when
      `code.db` is absent.
    - `pending_refresh_failure(project_root)` reads the marker for observability.
  - The watcher calls the retry wrapper; the warn log stays.
  - `wm_index_status` adds `code_index_stale` and `code_index_refresh_error`; the CLI prints the marker and clears it
    after a successful `wm index code`.

  ## Risks

  | Risk | Consequence | Mitigation |
  |---|---|---|
  | Marker never cleared | every read rebuilds forever | clear on every successful refresh path; recovery test |
  | Retry blocks the debouncer thread | delayed wiki events during a persistent lock | three attempts, sub-second total budget |
  | Marker mistaken for wiki-index stale flag | wrong status field | separate file and separate status keys |
  | Retrying a permanent error | wasted rebuilds and delayed reporting | `is_lock_contention` gate, fail-fast test |

  ## Verification

  - Red first: a test that makes `.wm/state/code.db` a directory forces a real failure, asserts the marker is written
    and readable, then repairs the DB and asserts the marker is cleared by a successful refresh.
  - Unit tests: retry stops at the attempt budget for lock errors, returns the first success, and does not retry a
    permanent error.
  - Marker edge cases: an absent index clears the marker; an unclearable marker (a directory at the marker path) does
    not fail a completed rebuild.
  - `cargo test -p wm-core --test code_index_watcher_test`, service unit tests, `cargo clippy` on touched crates.

  ## References

  - @wiki/tasks/refresh-code-index-drops-failed-refreshes-without-retry
  - @wiki/tasks/code-index-watcher-tests-have-a-load-dependent-deadline
  - @wiki/tasks/wiki-tool-reliability-wm-index-code-cannot-run-while-an-mcp-server-is-open-codedb-lock
  - @wiki/specs/code-index-cache-consumption
questions:
  - id: kind
    type: choice
    instructions: What kind of spec is this?
    options:
    - feature
    - system
    - doc
    - migration
    - experiment
  - id: scope
    type: choice
    instructions: How wide is the scope of this spec?
    options:
    - local
    - component
    - system
    - project-wide
  - id: status_class
    type: choice
    instructions: What lifecycle class is this spec in?
    options:
    - draft
    - reviewed
    - approved
    - superseded
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers:
  kind: system
  scope: local
  status_class: approved
  needs_tasks: true