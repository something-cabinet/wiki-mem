---
title: Refresh derived in-memory state at the write path
type: memory
tags: [architecture, graph, cache, stale]
status: active
---

schema_version: 1
state: |-
  If a write path mutates state that reads consult (graph/index), refresh the derived snapshot synchronously in the writer — never rely on a file watcher to catch up (wm-server runs no watcher; wm_task.get stayed stale until rebuild). Full: @wiki/patterns/refresh-derived-state-at-write-path
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
