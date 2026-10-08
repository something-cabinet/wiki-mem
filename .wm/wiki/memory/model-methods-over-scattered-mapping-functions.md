---
title: Model methods over scattered mapping functions
type: memory
tags: [decision, architecture, rust, enum]
status: active
---

schema_version: 1
state: |-
  Put enum to_str/from_str on the model, not in scattered standalone functions. Single match block eliminates drift, makes mapping discoverable via TypeName::. Full reference: @wiki/decisions/model-methods-over-scattered-mappings
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
