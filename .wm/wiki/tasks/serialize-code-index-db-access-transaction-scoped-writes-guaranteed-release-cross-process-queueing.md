---
title: 'Serialize code-index DB access: transaction-scoped writes, guaranteed release, cross-process queueing'
type: task
id: "wiki:tasks:serialize-code-index-db-access-transaction-scoped-writes-guaranteed-release-cross-process-queueing"
status: in-review
priority: high
tags: [code-intel, concurrency, transaction, architecture, reliability]
acceptance_criteria:
  - text: "Every write path runs inside an explicit transaction that rolls back on error, panic, or drop"
  - text: "Cross-process writers serialize without a daemon and either wait within a bound or surface the holder and remedy"
  - text: "In-process callers share one serialization point per canonical code.db path (or a documented, tested alternative)"
  - text: "rebuild_code_index cannot leave a partial index that the staleness probe reports as current"
  - text: "Tests cover rollback on failure, concurrent-process access, and lock release after every path"
relates_to:
  - {type: implements, target: wiki:specs:code-index-db-serialization}
implementation_notes: 'Implemented and verified 2026-10-20. rebuild_code_index now hashes via a short-lived handle, scans/extracts with no handle, then reopens to apply. apply_base_write wraps upsert + stale delete + DELETE resolved_edges in BEGIN IMMEDIATE; with_write_tx is the sole write path with commit-failure rollback and autocommit assertions. mark_refresh_started brackets every rebuild entry point; REBUILD_GUARD single-flights probe+rebuild with a post-acquire staleness re-check; metadata scans moved outside the handle; no-op rebuild skips edge rewrite (missing resolved edges are repaired); load_code_graph falls back to the filesystem snapshot on transient failures. Deviations: the guard is process-wide (one project per process), and the CLI fails fast with the holder/remedy instead of a 30s jittered wait. Gates: wm-code-intel 78+14, wm-core lib 230, watcher 13, e2e_code_intel 7, process-lock 1, wm-cli 5, mcp_test 116, graph_code_edges 5, clippy -D warnings, rustfmt. AC checkmarks still cannot be persisted (running MCP binary predates the check_ac fix).'
---

Finding (2026-10-20, raised while hardening the lock retry): the code index has no single serialization point.

- `CodeIndexDb` serializes calls only within one instance (`Arc<Mutex<InnerDb>>`); `graph/code_edges.rs`, `code_index_refresh_service.rs` and `wm-cli index code` each open short-lived instances, so in-process callers are not serialized with each other.
- Cross-process callers rely on turso's file lock and fail with `Locking error ... File is locked by another process` instead of waiting/queueing.
- `rebuild_code_index` is not atomic: `bulk_upsert_files` (own transaction) -> `delete_stale_files` -> `replace_resolved_edges` (own transaction). A failure between steps leaves a partial index that the hash/mtime probe can report as current.
- Transactions are rolled back manually on the Err path only; there is no RAII guard and no busy-wait.

Proposal under review: every DB action acquires a transaction lock, other actions queue for relay, and the transaction always releases the lock. A spec and an architecture decision (oracle + turso 0.7 capability check) are in flight; do not implement before the spec is approved.