---
title: Index rebuild reports success when vectors cannot persist
type: task
id: "wiki:tasks:index-rebuild-reports-success-when-vectors-cannot-persist"
status: done
priority: high
tags: [index, embeddings, tool-reliability]
spec: wiki:specs:index-rebuild-must-fail-when-vectors-cannot-persist
acceptance_criteria:
  - text: "With vectors.db held open by another process, index rebuild and index embed exit non-zero without running the embedder and name the lock as the reason"
    checked: true
  - text: "A persist failure after embedding returns an error and never prints Rebuild complete"
    checked: true
  - text: "A successful rebuild changes vectors.db, verified by reading it back in a test"
    checked: true
  - text: "index rebuild --skip-embed still succeeds while the lock is held"
    checked: true
implementation_notes: |-
  2026-11-02: with the MCP server (pid 14267) stopped, the same wm-cli 0.6.1 index rebuild in eightcap-new-portal persisted vectors: vectors.db grew 819200 -> 1478656 bytes, vectors.db-wal 2702752 -> 3687432, no persist warning. Confirms the lock is the trigger. Spec approved; next step is the failing tests.
  Implemented: VectorStore keeps the open error (db_error) and exposes ensure_persistable; wm_index_rebuild and wm_index_embed check it before embedding and return an error naming the cause; a persist or embedding failure is now an error instead of a logged warning; index rebuild and index embed propagate tool errors so the CLI exits 1. Measured in eightcap-new-portal with the lock held: exit 1 in 0.9 s with the lock named, versus 38 s wall, 294 s CPU, Rebuild complete and exit 0 before. Tests, red first: wm-embed (2), wm-tools index.rs (5), apps/wm-core/tests/index_rebuild_test.rs (4). Open: criterion 3 (a test proving a successful rebuild changes vectors.db) needs an embedder that can be injected into EngineState; IndexState takes it only in its constructor, so it is verified by hand only (vectors.db 819200 -> 1478656 bytes after a full rebuild with the lock released).
  Criterion 3 closed by packages/wm-tools/tests/index_persist_test.rs: a rebuild with a loaded counting embedder writes vectors that load_all_raw reads back from the database, and a store that cannot persist fails wm_index_rebuild and wm_index_embed before the embedder is called.
---

Reproduced in ~/projects/eightcap-new-portal with wm-cli 0.6.1 while an MCP server held vectors.db: index rebuild took 38 s wall and 294 s CPU, printed Rebuild complete, exited 0, and left vectors.db unchanged. The only signal was WARN Failed to persist vectors to turso: no turso database configured. Cause chain: lock at load, VectorStore::new drops the open error and sets db None, the handler logs the save error and returns success. Violates rule durable-write-or-error. Companion to the quiet-warnings task. See the spec.