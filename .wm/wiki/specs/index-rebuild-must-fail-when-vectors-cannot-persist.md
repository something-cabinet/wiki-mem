---
title: Index rebuild must fail when vectors cannot persist
type: spec
id: "wiki:specs:index-rebuild-must-fail-when-vectors-cannot-persist"
status: approved
tags: [index, embeddings, tool-reliability, spec]
acceptance_criteria:
  - text: With vectors.db held by another process, index rebuild and index embed exit non-zero before embedding and name the lock
  - text: A persist failure after embedding returns an error and never prints Rebuild complete
  - text: A successful rebuild changes vectors.db, verified by a read-back test
---

schema_version: 1
state: |-
  ## Goal

  `wm-cli index rebuild` and `wm_index_embed` must never report success when the vectors were not written to vectors.db, and must say why.

  ## Verified current state

  Reproduced with wm-cli 0.6.1 in ~/projects/eightcap-new-portal while another wm-cli process (an MCP server, pid 14267) held vectors.db open:

  - `wm-cli index rebuild` ran 38 s wall, 294 s CPU, printed "Rebuild complete" and exited 0.
  - vectors.db and vectors.db-wal kept the same size and mtime, so nothing was persisted.
  - The only signal was `WARN wm_tools::tools::index: Failed to persist vectors to turso: no turso database configured`.

  ## Causes

  1. apps/wm-core/src/engine/main_engine_factory.rs: when VectorStore::load_from_disk fails with the lock error, it logs "starting fresh" and calls VectorStore::new.
  2. packages/wm-embed/src/services/vector_store_service.rs: VectorStore::new retries VectorDb::open, discards the error with `.ok()`, and returns a store with `db: None`.
  3. packages/wm-tools/src/tools/index.rs lines 75-77 and 137-139: after embedding everything, save_to_disk fails with "no turso database configured", the handler logs a warning and still returns the embed count as success.
  4. The message hides the real cause, the lock held by another process.

  Violates @wiki/rules/durable-write-or-error.

  ## Change

  - Keep the open error in the store (db unavailable with its reason) instead of dropping it.
  - index rebuild and index embed check that the store can persist before embedding, and fail fast with the lock reason and the holding process hint.
  - If a persist fails after embedding, return an error from the tool and exit non-zero, never "Rebuild complete".
  - `--skip-embed` stays valid while the lock is held.

  ## Verification

  - Test: with vectors.db held open by a second handle, index rebuild exits non-zero in under one second without running the embedder and names the lock.
  - Test: after a successful rebuild, vectors.db content differs from before (read back).
  - Manual: repeat the eightcap-new-portal reproduction.
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
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers:
  kind: system
  scope: component
  needs_tasks: true