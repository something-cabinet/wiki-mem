---
title: Code Index DB Serialization — short lock windows, atomic writes, crash-safe rebuilds
type: spec
id: "wiki:specs:code-index-db-serialization"
status: approved
tags: [code-intel, concurrency, transaction, architecture, spec]
acceptance_criteria:
  - checked: false
    text: Spec reviewed and its requirements are implemented
---

schema_version: 1
state: |-
  ## Goal

  Make code-index DB access correct under concurrency without reintroducing a daemon: shorten how long the file lock
  is held, make each logical write atomic, make a crash mid-rebuild fail safe, and serialize rebuilds in-process.
  A relay queue is explicitly rejected (see Decision).

  ## Verified current state

  - `CodeIndexDb` wraps one turso connection in `Arc<Mutex<InnerDb>>`; a single instance serializes its calls, but
    `graph/code_edges.rs`, `code_index_refresh_service.rs` and `wm-cli index code` each open short-lived instances.
  - turso 0.7 takes a **non-blocking exclusive `fcntl` lock at open** and releases it when the `Database` drops
    (`turso_core-0.7.0/io/unix.rs:278-309`). `Connection::busy_timeout` exists
    (`turso-0.7.0/src/connection.rs:240`) but cannot cover the open-time `LockingError`.
  - `rebuild_code_index` held the `CodeIndexDb` across `code_walk`, file reads, hashing, and tree-sitter extraction
    (`packages/wm-code-intel/src/services/ingest_service.rs:17-114`). That was the dominant lock-hold window.
  - The rebuild spanned three transactions: `bulk_upsert_files`, `delete_stale_files`, `replace_resolved_edges`.
    A failure between them left new base data with old resolved edges, which the hash/mtime probe could report as
    current.
  - `turso::Transaction::drop` does not roll back immediately; it stores `dangling_tx` and rolls back on the next
    connection access (`turso-0.7.0/src/transaction.rs:219-226`).
  - `BEGIN IMMEDIATE` is available, and turso dedupes `Database` per file in-process via `DATABASE_MANAGER`.
  - Multiprocess WAL exists but is opt-in, experimental, and mixed-mode opens can silently lose writes (turso issue
    #9362).

  ## Decision

  | Proposal clause | Verdict |
  |---|---|
  | Every DB action is a transaction lock | **Partial** — every logical multi-statement write is one transaction; reads are not transactional. |
  | Others are pushed into a relay queue | **Rejected** — no daemon; cross-process ordering is the DB write lock plus a bounded caller retry. An in-process queue is a mini-daemon and contradicts the recorded sync-write decision. |
  | The transaction always releases the lock | **Adopt** — via an explicit `with_write_tx` helper; commit on success, rollback on error and on commit failure. |

  ## Functional requirements

  - **FR-1 (lock window)** Read file hashes, then drop the DB handle; walk, read, hash, and extract with no handle;
    reopen only to apply writes. Do the same for `scan_file_metadata` in `refresh_if_stale` / `index_lag_seconds`.
  - **FR-2 (atomic base write)** `bulk_upsert_files` + `delete_stale_files` + `DELETE FROM resolved_edges` run in one
    `BEGIN IMMEDIATE` transaction. The derived graph is therefore atomically marked incomplete with the base change.
  - **FR-3 (crash intent)** Write the stale marker before the base mutation on every rebuild entry point; clear it
    only after the resolved-edge replacement commits. A crash leaves base data + an empty resolved table, and the
    loader's existing "no resolved edges" fallback recomputes.
  - **FR-4 (transaction helper)** One async `with_write_tx` path: `BEGIN IMMEDIATE`, explicit `COMMIT`, explicit
    `ROLLBACK` on error, and `ROLLBACK` when `COMMIT` fails. `debug_assert!(conn.is_autocommit())` after each
    helper call.
  - **FR-5 (single-flight)** A process-wide rebuild guard around probe+rebuild with a double-check after acquiring.
    One project per process is the current invariant; key the guard by canonical db path if multi-root composition
    lands.
  - **FR-6 (skip no-op work)** Skip edge resolution when nothing changed, except when resolved edges are missing
    (crash repair).
  - **FR-7 (wait policy)** Keep the watcher's bounded retry for lock errors; the CLI rebuild fails fast with the
    holder and remedy rather than blocking for 30 s; MCP read paths degrade to the filesystem snapshot instead of
    failing the tool call.
  - **FR-8 (graceful reads)** `load_code_graph` serves recomputed or last-known data on a transient lock and logs a
    warning.

  ## Non-functional requirements

  - **NFR-1** No daemon, no actor thread, no channel, no global connection cache.
  - **NFR-2** Do not hold the DB lock across CPU work (parsing, hashing, resolution).
  - **NFR-3** No comments, no `else`, no `#[allow(...)]`, named constants, functions at most 20 lines, TDD.
  - **NFR-4** Existing public behavior and JSON shapes stay additive-only; all current tests stay green.

  ## Implementation (landed)

  - `rebuild_code_index(db_path, project_root, force)`: open for hashes, drop, scan/extract, reopen, apply.
  - `CodeIndexDb::apply_base_write` wraps upsert + stale delete + resolved-edge invalidation in `with_write_tx`;
    `with_write_tx` is the only write path, with commit-failure rollback.
  - `mark_refresh_started` writes the intent marker; every success path clears it.
  - `REBUILD_GUARD` (poison-recovering) single-flights probe+rebuild; staleness is re-checked after acquiring.
  - Metadata scans run before opening the DB; `busy_timeout` is 5 s as a future multiprocess defense.
  - `load_code_graph` falls back to `CodeIndexSnapshot::collect_from_fs` on refresh/open/read errors.

  ## Risks

  | Risk | Consequence | Mitigation |
  |---|---|---|
  | New base + stale resolved served | wrong code-intel answers | delete resolved in the base tx; loader fallback; injected-failure tests |
  | Long transaction holds the write lock | slow rebuild, WAL spike | per-step transactions, not one giant one |
  | Schema DDL runs on every open | wider lock window, swallowed ALTER errors | future: schema-version gate; not changed here |
  | Two in-process rebuilds interleave | lost updates | process-wide guard + double-check |
  | Multiprocess WAL mixed versions | silent write loss | opt-in only; version gate; issue #9362 |

  ## Out of scope

  - Relay queue, durable queue table, actor thread, daemon, connection registry/cache.
  - Wrapping reads in write transactions; `BEGIN EXCLUSIVE`.
  - Enabling multiprocess WAL before a probe and a rebuild lease exist.
  - Per-path (multi-root) guard keying until multi-root composition lands.

  ## Verification

  - `cargo test -p wm-code-intel`: 78 unit + 14 integration pass.
  - `cargo test -p wm-core --lib`: 230 pass; `code_index_watcher_test`: 13 pass; `e2e_code_intel`: 7 pass;
    `code_db_process_lock_test`: 1 pass after rebuilding `wm-cli`.
  - `wm-cli` 5 tests and `wm-tools` `mcp_test` 116 tests pass; `graph_code_edges` 5 pass.
  - Injected failure in `apply_base_write` rolls back to (old base + old resolved); a successful base write leaves
    (new base + zero resolved).
  - No-op rebuild preserves resolved edges; missing resolved edges are repaired.
  - `cargo clippy -p wm-code-intel -p wm-core -p wm-cli -- -D warnings` clean; rustfmt clean on all touched files.

  ## References

  - @wiki/tasks/serialize-code-index-db-access-transaction-scoped-writes-guaranteed-release-cross-process-queueing
  - @wiki/decisions/code-index-db-serialization
  - @wiki/tasks/wiki-tool-reliability-wm-index-code-cannot-run-while-an-mcp-server-is-open-codedb-lock
  - @wiki/patterns/durable-stale-marker-for-failed-refresh
  - @wiki/memory/a-process-wide-cached-connection-is-a-process-wide-file-lock
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
  scope: component
  status_class: approved
  needs_tasks: true