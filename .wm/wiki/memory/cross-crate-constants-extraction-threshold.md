---
title: Cross-crate constants extraction threshold
type: memory
tags: [pattern, constants, architecture]
status: active
---

schema_version: 1
state: |-
  Magic values used in 3+ crates → `wm-constants` shared package (zero deps). Used in 1-2 crates → per-crate `constants/` dir with barrel `mod.rs`. Full pattern: @wiki/patterns/cross-crate-constants
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
