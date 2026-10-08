---
title: Materialize expensive passes at index time, not query time
type: memory
tags: [pattern, architecture, performance]
status: active
---

schema_version: 1
state: |-
  Run expensive global analysis (symbol resolution, type inference) once at index time and persist results. Query paths read pre-computed data from a `resolved_edges` table. Fallback to on-the-fly when table is empty. Full reference: @wiki/patterns/materialize-expensive-passes-at-index-time
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
