---
title: "Zero Comments — Extract Over Document"
type: memory
tags: [naming, quality, rule]
created_at: "2026-07-24"
relates_to:
  - {type: references, target: wiki:decisions:zero-comments-extract-over-document}
---

schema_version: 1
state: |-
  No `//`, `///`, `//!`, `/** */` in source. If it needs a comment, split/rename instead. TODOs → WM tasks. Enforce via `rg '^\s*//'`. Full reference: @wiki/decisions/zero-comments-extract-over-document
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
