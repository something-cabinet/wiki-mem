---
title: 'Decision: Open vectors.db per operation and wait on lock errors'
type: decision
id: "wiki:decisions:open-vectors-db-per-operation"
status: draft
relates_to:
  - {type: references, target: wiki:tasks:index-rebuild-reports-success-when-vectors-cannot-persist}
  - {type: extends, target: wiki:decisions:code-index-db-serialization}
  - {type: references, target: wiki:patterns:verify-writes-by-read-back}
---

schema_version: 1
state: |-
  ## Context

  `VectorStore` kept one turso connection for the whole life of the engine. turso takes a non-blocking exclusive lock when a database is opened and holds it until the handle drops, so a long-lived `wm-cli mcp` blocked every other `wm-cli` process from opening `.wm/state/vectors.db` (#146). The engine factory then logged `starting fresh` and ran with no database. Writes still exited 0 and wrote the markdown, and the vectors and edges that live only in the database were lost.

  `wiki:decisions:code-index-db-serialization` records the same turso behavior for `code.db`: `busy_timeout` cannot cover the open-time lock, and turso dedupes one `Database` per file inside a process.

  ## Decision

  `VectorStore` keeps only the database path. It opens `vectors.db` for each load, save, probe, search, upsert and removal, and closes it afterwards. A `Locking error` on open is retried every 50 ms for up to 5 s for writes (`VECTOR_DB_WRITE_WAIT_MS`). Searches try once and fall back to the in-memory vectors.

  `wm_page` create and update read the database back (`VectorStore::unpersisted_sections`) and return `vectors_persisted`, plus `vectors_warning` when it is false.

  ## Rationale

  The vectors are cached in memory, so the database is only needed at startup and when saving. Holding the lock between those moments gains nothing and blocks other processes. turso's own wait does not apply at open, so the retry is ours. The read-back follows `wiki:patterns:verify-writes-by-read-back`.

  ## Consequences

  - Two processes can write in turn. A full save deletes rows that are not in memory. This is safe because `index rebuild` and `index embed` embed the whole corpus first, but two rebuilds racing are untested.
  - The `starting fresh` fallback in `main_engine_factory.rs` still exists. It now triggers only after the 5 s wait fails.
  - Verified on macOS with `wm-cli mcp` running: `status`, `search`, `index rebuild --skip-embed`, `index embed --force` and `index rebuild` exited 0 and the vectors were in the database. Not tested on Windows.
  - Commits: `97ccc6b`, `d9c44b6`, `be14398`.

  ## Related

  - #146
  - `wiki:decisions:code-index-db-serialization`
  - `wiki:patterns:test-file-locks-with-a-second-process`

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
answers: {}