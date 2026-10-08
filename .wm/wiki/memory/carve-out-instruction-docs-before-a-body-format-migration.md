---
title: Carve out instruction docs before a body-format migration
type: memory
tags: [migration, wiki, agent-workflow, search, graph]
status: active
---

schema_version: 1
state: |-
  Before a bulk doc-body format migration, carve out agent-instruction/recall pages (rule, core, memory, task, spec, index, steering) or you break instruction loading; and make search/graph record-aware (read the record `state` for sections/tags/refs) or BM25 ranking and body-link edges regress. Full: @doc/patterns/instruction-doc-carve-out-before-body-migration
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
