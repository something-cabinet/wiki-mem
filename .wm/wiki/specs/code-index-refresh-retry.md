---
title: Code Index Refresh Retry — bounded retry + durable stale marker
type: spec
id: "wiki:specs:code-index-refresh-retry"
status: approved
tags: [code-intel, reliability, observability, spec]
---

schema_version: 1
state: |-
  ## Goal

  Stop `refresh_code_index_for` from dropping a failed watcher-triggered refresh silently. A failure must be retried
  within a bound, and when the bound is exhausted the index must be marked stale durably so the next reader rebuilds,
  with the failure visible to callers instead of a single log line.

  ## Verified current state

  - `apps/wm-core/src/engine/main_engine_factory.rs:141` — `refresh_code_index_for` matches `Err(e)` and only calls
    `tracing::warn!`. No retry, no state change, no return value.
  - `apps/wm-core/src/engine/main_engine_factory.rs:334` — the watcher thread calls it once per debounced batch.
  - `apps/wm-core/src/engine/code_index_refresh_service.rs:15,37` — `refresh_code_index` / `refresh_if_stale` return
    `Result` and detect staleness only by comparing on-disk file count/mtime against stored hashes.
  - `packages/wm-tools/src/tools/index.rs:200` — `wm_index_status` is the caller-facing surface for code-index
    state (`code_index_age_seconds`).

  A transient lock collision therefore loses the event. When the failed rebuild partially wrote hashes, the mtime
  comparison can even report the index as current, so the mtime-only staleness probe is insufficient after a failure.

  ## Functional requirements

  - **FR-1** A failed refresh is retried with bounded exponential backoff before being declared lost.
  - **FR-2** When the retry budget is exhausted, a durable stale marker is written under `.wm/state/` so
    `refresh_if_stale` rebuilds on the next read even if file metadata comparisons say the index is current.
  - **FR-3** A successful refresh (watcher, `refresh_if_stale`, or `wm index code`) removes the marker.
  - **FR-4** The marker is exposed through `wm_index_status` (and the CLI code-index output), not only logs.

  ## Non-functional requirements

  - **NFR-1** Bounded blocking: total retry sleep stays under one second on the watcher thread.
  - **NFR-2** Existing JSON shapes stay additive-only.
  - **NFR-3** Project rules apply: no comments, no `else`, no `#[allow(...)]`, named constants, functions at most
    20 lines, TDD with the failing test written first.

  ## Design

  - Add `CODE_INDEX_STALE_FILE` to `packages/wm-constants/src/files.rs`.
  - In `code_index_refresh_service`:
    - `refresh_code_index_with_retry(project_root)` runs the refresh up to `REFRESH_ATTEMPTS` times with
      `REFRESH_BACKOFF * 4^attempt` sleeps, and on exhaustion writes the error text to the marker file.
    - `is_stale` returns true while the marker exists, so `refresh_if_stale` rebuilds.
    - `refresh_code_index` and `refresh_if_stale` clear the marker after a successful rebuild.
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

  ## Verification

  - Red first: a test that makes `.wm/state/code.db` a directory forces a real failure, asserts the marker is written
    and readable, then repairs the DB and asserts the marker is cleared by a successful refresh.
  - Unit test for the retry bound (attempt count and exhaustion).
  - `cargo test -p wm-core --test code_index_watcher_test`, service unit tests, `cargo clippy` on touched crates.

  ## References

  - @wiki/tasks/refresh-code-index-drops-failed-refreshes-without-retry
  - @wiki/tasks/code-index-watcher-tests-have-a-load-dependent-deadline
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
