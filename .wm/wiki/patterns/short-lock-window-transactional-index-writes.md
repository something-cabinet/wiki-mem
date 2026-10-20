---
title: 'Pattern: Short lock windows with transactional derived-index writes'
type: pattern
id: "wiki:patterns:short-lock-window-transactional-index-writes"
status: reviewed
tags: [pattern, concurrency, transaction, index, architecture]
relates_to:
  - {type: relates_to, target: wiki:patterns:durable-stale-marker-for-failed-refresh}
---

schema_version: 1
state: |-
  ## Problem

  A local index database (turso/SQLite) takes a process-wide advisory lock when opened, and releases it only when
  the handle drops. A rebuild that holds the handle while walking the tree, reading files, hashing, and running
  tree-sitter extraction therefore blocks every other process for the whole CPU phase — the lock error names the
  database, but the duration is the defect. Separately, a multi-step write (upsert base, delete stale, replace
  derived edges) spread across separate transactions can crash into new base data plus stale derived data.

  ## Solution

  Four rules, in order of value:

  1. **Shorten the hold window.** Open only for the cheap reads (hashes, counts), drop the handle, do all CPU work
     with no handle, reopen only to apply writes. Do metadata walks before opening. A handle is never held across
     parsing, hashing, or resolution.
  2. **One transaction per logical write.** Wrap every multi-statement mutation in a single `BEGIN IMMEDIATE`
     transaction through one helper (explicit commit; explicit rollback on error and on commit failure). Do not wrap
     reads in write transactions.
  3. **Invalidate derived data in the same transaction as its base data.** `DELETE FROM resolved_edges` runs inside
     the base transaction. A crash then leaves either (old base + old derived) or (new base + empty derived), never
     (new base + old derived). Pair this with a reader fallback that recomputes when the derived table is empty.
  4. **Write an intent marker before mutating and clear it after success.** The marker is the durable "index may be
     inconsistent" bit; readers treat it as stale. A cleanup failure must be logged and must not fail the rebuild.

  Supporting rules: a single-flight guard (re-check staleness after acquiring) stops two in-process rebuilds from
  interleaving; skip derived re-materialization when nothing changed unless the derived table is missing; surface the
  lock holder and remedy on failure.

  ## When to Use

  - Any local database with process-wide file locking and a rebuild that mixes CPU work with writes.
  - Multi-step writes that update base rows and derived/materialized rows.
  - Single-user tools where a daemon is deliberately absent and SQLite/turso is the only coordination point.

  ## When Not to Use

  - Multi-writer server databases where transactions are already managed by a pool or ORM and reads must be
    MVCC-snapshotted; the per-process file-lock argument does not apply.
  - Workloads where derived data must be atomically visible at all times; the empty-derived fallback is the trade-off
    that makes per-step transactions acceptable.

  ## Failure Mode This Prevents

  `rebuild_code_index` held `CodeIndexDb` across the entire scan (seconds on a large repo), so a second process
  failed at open with `Locking error ... code.db-wal`. The rebuild also spanned three transactions, so a failure
  between base and edge replacement left new base rows with old resolved edges, which the mtime/hash probe reported
  as current.

  ## Related

  - @wiki/decisions/code-index-db-serialization — why a relay queue was rejected
  - @wiki/specs/code-index-db-serialization — verification and phase plan
  - @wiki/patterns/durable-stale-marker-for-failed-refresh — what to do when the refresh still fails
  - @wiki/patterns/refresh-derived-state-at-write-path — synchronous refresh at the write path
  - @wiki/tasks/serialize-code-index-db-access-transaction-scoped-writes-guaranteed-release-cross-process-queueing
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
  problem_kind: architecture
  preconditions_required: true
  complexity: moderate
  language_specific: false