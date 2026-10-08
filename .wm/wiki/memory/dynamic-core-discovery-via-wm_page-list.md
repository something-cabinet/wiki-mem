---
title: Dynamic core discovery via wm_page.list
type: memory
tags: [decision, init, core]
status: active
---

schema_version: 1
state: |-
  wm-init now discovers core pages dynamically via wm_page.list({'type': 'core'}) instead of hardcoded IDs. README loaded explicitly. Full reference: @wiki/decisions/dynamic-core-discovery-over-hardcoded-ids
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
