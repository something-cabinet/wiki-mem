---
title: 'Wiki Tool Reliability: wm index code cannot run while an MCP server is open (code.db lock)'
type: task
id: "wiki:tasks:wiki-tool-reliability-wm-index-code-cannot-run-while-an-mcp-server-is-open-codedb-lock"
status: done
priority: high
tags: [tool-reliability, code-intel, concurrency, lock]
acceptance_criteria:
  - text: "wm index code either succeeds while an MCP server is running, or fails with an error that names the holding process and the remedy"
  - text: "The locking behaviour is explained: why the DB open takes an exclusive lock when SQLite WAL should allow concurrent readers"
  - text: "A test covers the concurrent case (server holding the DB while an index command runs)"
  - text: "The chosen approach is recorded as a decision, since it changes how two shipped entry points interact"
implementation_notes: |2-

  ## FIXED and independently verified (2026-10-15)

  **Root cause:** `apps/wm-core/src/graph/code_edges.rs` cached an `Arc<CodeIndexDb>` in a process-wide `static OnceLock` (`CODE_DB_CACHE`). `CodeIndexDb` holds a turso connection, so `.wm/state/code.db` and `code.db-wal` stayed open for the server`s entire process lifetime and a separate `index code` process could never take the lock. `CodeIndexDb` exposes no `close()`/`unlock()` — the lock is released only when the connection is dropped — so the only way to express unlock-after-use was to stop holding it. The cache existed solely to avoid one re-open, while `load_code_graph` already re-queries `load_resolved_edges()` on every call and `refresh_if_stale` already opens its own connection.

  **Fix:** the static and `CachedCodeDb` are deleted and `open_code_index` opens and returns a short-lived `CodeIndexDb` (`Option<CodeIndexDb>` instead of `Option<Arc<CodeIndexDb>>`). No retry loop and no holder message were added, because the lifetime hold was the defect; a retry would have been a compensating layer over a cache that should not exist.

  **Regression test:** `apps/wm-core/tests/code_db_process_lock_test.rs` seeds a temp project, creates the DB through a child `wm-cli index code`, calls `load_code_graph` **in the test process** to populate what was the cache, then spawns `index code` again and asserts a clean exit. It failed with the reported `Locking error` before the fix and passes after. A second OS process is required because the defect is ownership of an OS file lock outliving a call, which no in-process assertion can observe.

  **CORRECTION to the framing used when this was assigned.** The brief claimed opens per query would go from two to one. That was wrong and the implementing lane caught it: `load_code_graph` opens twice per call — the staleness probe and the graph read — both before and after. The open **count is unchanged**; what changed is the **lifetime**, from process-lifetime to per-call. The gain is the invariant, not a saved open.

  **Verified by the reviewer, not from the lane report:** `CODE_DB_CACHE` gone workspace-wide; `open_code_index` has no caller other than `load_code_graph`; the two-process test passes; the 33 CLI goldens, `cli_test` (17) and the watcher suite (7) are green.

  **Residual, deliberately not addressed:** two *genuinely simultaneous* `index code` processes can still contend on turso`s exclusive file lock and one may fail with the same error. That is a serialization concern rather than the lifetime-hold defect. A bounded retry with an explicit message would be legitimate there as a separate, clearly-scoped change.

  **Related observation, unrelated to the lock:** `run_async` in `code_index_db.rs` builds a fresh `tokio::runtime::Runtime` per database call when there is no ambient runtime, so every one-shot CLI code operation pays runtime construction.

  **API note:** `open_code_index` is `pub` and re-exported, so its signature changed. No in-repo caller other than `load_code_graph` needed updating.
---

**Bug description:** `wm-cli index code` fails outright when any MCP server process is running, because the server holds an exclusive lock on the code index database. The two primary entry points of the tool cannot coexist.

**Tool name:** `wm-cli index code` (also blocks any command opening `code.db`, including the staleness-refresh path).

**Full input parameters:** `./target/debug/wm-cli index code`

**Full error output:**
```
Error: Failed to open code db: Locking error: Failed locking file
'/Users/nhkhanh/projects/wiki-mem/.wm/state/code.db-wal'. File is locked by another process
```

**Counter-evidence / holder identified:** `lsof .wm/state/code.db .wm/state/code.db-wal` shows
```
wm-cli  86795  nhkhanh  12u  REG  .wm/state/code.db
wm-cli  86795  nhkhanh  13u  REG  .wm/state/code.db-wal
```
and `pgrep -fl wm-cli` identifies PID 86795 as `wm-cli mcp` — the MCP server serving the agent session in which the index command was attempted.

**Reproduction steps:**
1. Start an MCP server (`wm-cli mcp`) — the normal state during any agent session.
2. In another shell run `wm-cli index code`.
3. Observe the locking error; the index is not refreshed.

**Impact:**
- An agent session cannot refresh the code index while it is running, which is exactly when it most needs to — the index becomes stale for the whole session.
- It blocked a live verification of the gitignore read/index policy (a gitignored `tmp/probe/probe.rs` could not be indexed to prove the violation), so an unrelated investigation was stalled.
- The message names neither the holding process nor a remedy, so the user cannot tell whether the lock is stale, transient, or held by their own session.

**Workaround:** stop the MCP server, run `index code`, restart it — impractical mid-session, and it discards the session.

**Fix direction (needs a decision):** SQLite WAL normally permits concurrent readers with one writer, so first determine why the open takes an exclusive lock. Then either (a) let `index code` proceed against the shared database, (b) retry with bounded backoff and report the holder, or (c) release/hand off the lock in the server between tool calls. Whatever is chosen, the error message must name the holding process when a lock cannot be taken.

**Related:** `@wiki/tasks/wmcode-read-policy-should-follow-gitignore-not-a-hardcoded-hidden-file-ban` (the verification this blocked) and `@wiki/decisions/code-read-policy-follows-gitignore`.