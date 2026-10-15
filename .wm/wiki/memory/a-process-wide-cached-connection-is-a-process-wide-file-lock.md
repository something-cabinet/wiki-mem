---
title: A process-wide cached connection is a process-wide file lock
type: memory
tags: [concurrency, lock, code-intel, cache]
status: active
---

schema_version: 1
state: |-
  `apps/wm-core/src/graph/code_edges.rs` cached an `Arc<CodeIndexDb>` in a static `OnceLock`, so the turso connection kept `.wm/state/code.db` and `code.db-wal` open for the server's entire lifetime. No other process could open the index: `wm-cli index code` failed with `Locking error ... File is locked by another process` whenever an MCP server was running — exactly when the index most needs refreshing.

  `CodeIndexDb` exposes no close/unlock and the connection releases on drop, so unlock-after-use is only expressible as not holding it. Deleting the cache was the fix; a retry loop would have been a compensating layer over a cache that should not exist.

  The cache saved a single re-open while costing exclusive ownership, and `load_code_graph` already re-queried resolved edges on every call.

  Correction worth carrying: removing it did NOT reduce the open count. There are two short-lived opens per call before and after (the staleness probe, then the graph read). Only the LIFETIME changed, from process-lifetime to per-call.

  Verified by a two-process test, because an in-process assertion cannot observe an OS file lock that outlives a call.

  Refs: @wiki/tasks/wiki-tool-reliability-wm-index-code-cannot-run-while-an-mcp-server-is-open-codedb-lock
questions:
  - id: layer
    type: choice
    instructions: Which memory layer does this entry belong to?
    options:
    - project
    - global
    - session
  - id: store_or_skip
    type: noul
    instructions: This entry is worth storing as durable memory.
  - id: dedup_action
    type: choice
    instructions: How should this entry relate to existing memory?
    options:
    - new
    - merge
    - supersede
    - skip
  - id: confidence
    type: score
    instructions: How confident is the recorded knowledge?
    levels:
    - low
    - medium
    - high
answers: {}
