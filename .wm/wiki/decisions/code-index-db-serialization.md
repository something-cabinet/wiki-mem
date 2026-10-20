---
title: 'Decision: Serialize code-index DB access without a relay queue'
type: decision
id: "wiki:decisions:code-index-db-serialization"
status: approved
tags: [code-intel, concurrency, transaction, architecture, decision]
relates_to:
  - {type: references, target: wiki:tasks:serialize-code-index-db-access-transaction-scoped-writes-guaranteed-release-cross-process-queueing}
---

schema_version: 1
state: |-
  ## Context

  Concurrent access to `.wm/state/code.db` produced `Locking error: Failed locking file 'code.db-wal'. File is
  locked by another process` between a long-lived `wm-cli mcp` and one-shot `wm-cli` commands. A proposal asked for
  every DB action to take a transaction lock, with other actions queued for relay, and a guaranteed release.

  Verified against the vendored turso 0.7.0 source:

  - `CodeIndexDb::open` takes a non-blocking exclusive `fcntl` lock and holds it until the `Database` drops
    (`turso_core-0.7.0/io/unix.rs:278-309`). `Connection::busy_timeout` exists but cannot cover that open-time error.
  - `rebuild_code_index` holds the handle across the walk, file reads, hashing, and tree-sitter extraction — seconds
    on a large repo. That hold time, not the transaction count, is the dominant cause.
  - The rebuild spans three transactions (`bulk_upsert_files`, `delete_stale_files`,
    `replace_resolved_edges`), so a crash between them leaves new base data with old resolved edges.
  - `turso::Transaction::drop` only stores `dangling_tx` and rolls back on the next connection access.
  - turso dedupes one core `Database` per file in-process (`DATABASE_MANAGER`), and `BEGIN IMMEDIATE` is available.
  - Multiprocess WAL is opt-in and experimental (`Builder::experimental_multiprocess_wal`).

  ## Decision

  **Adopted:** shorten the lock window, use one transaction per logical write with an explicit `with_write_tx`
  helper, make the base mutation atomic with `DELETE FROM resolved_edges`, pre-write an intent marker before
  mutation, and serialize rebuilds in-process with a single-flight guard.

  **Rejected:** a relay queue, durable queue table, actor thread, daemon, or global connection registry. There is no
  single owner to relay to without a daemon, the DB's write lock is already the cross-process queue, and an
  in-process queue contradicts the recorded sync-write decision.

  **Deferred:** multiprocess WAL. It would make `busy_timeout` the queue and allow concurrent readers, but it is
  experimental, rejects mixed-mode opens, and removes free rebuild exclusion (a rebuild lease would be required).

  ## Rationale

  - The observed failures are caused by lock-hold duration; shortening it removes the contention window that no
    amount of transaction wrapping can remove.
  - Atomicity belongs to logical writes, not to every action. Wrapping reads in write transactions would serialize
    reads for no integrity gain.
  - An explicit helper is required because `Transaction::drop` does not release immediately; the current manual
    rollback also misses the commit-failure path.
  - Deleting resolved edges inside the base transaction makes "new base + old resolved" unrepresentable, and the
    loader's empty-resolved fallback already recomputes from base.

  ## Consequences

  - Rebuilds are atomic in two committed steps with an intent marker, not one giant transaction; readers may
    recompute during the gap, which is acceptable for a local single-user tool.
  - CLI rebuilds may wait with jitter and report the holder; MCP reads fail fast and serve recomputed or last-known
    data.
  - Multiprocess WAL remains a future opt-in gated on a probe, a rebuild lease, and a mixed-version error.
  - No new daemon or queue lifecycle exists to operate or crash-recover.

  ## Related

  - @wiki/specs/code-index-db-serialization
  - @wiki/tasks/serialize-code-index-db-access-transaction-scoped-writes-guaranteed-release-cross-process-queueing
  - @wiki/decisions/code-index-cache-architecture
  - @wiki/patterns/durable-stale-marker-for-failed-refresh
questions:
  - id: outcome
    type: choice
    instructions: What is the recorded outcome of this decision?
    options:
    - adopted
    - rejected
    - deferred
    - superseded
    - abandoned
  - id: reversibility
    type: noul
    instructions: The decision can be reversed cheaply without data migration or cross-module breakage.
  - id: confidence
    type: score
    instructions: How strong is the recorded justification for the selected outcome?
    levels:
    - low
    - medium
    - high
  - id: impact
    type: choice
    instructions: How wide is the blast radius of this decision?
    options:
    - local
    - component
    - system
    - project-wide
answers:
  outcome: adopted
  reversibility: false
  confidence: high
  impact: component