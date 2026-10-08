---
title: Identical-function → generic composition pattern
type: memory
tags: [pattern, refactoring, boilerplate, rust]
status: active
---

schema_version: 1
state: |-
  When 3+ functions share identical structure with only data varying, extract a private generic fn. Each variant becomes a thin data-only wrapper. Saves ~120 lines from symbols_helper. Full reference: @wiki/patterns/identical-function-composition
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
