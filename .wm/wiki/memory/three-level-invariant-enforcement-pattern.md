---
title: Three-level invariant enforcement pattern
type: memory
tags: [pattern, invariant]
status: active
---

schema_version: 1
state: |-
  When enforcing a project invariant, use three levels: (1) Architecture — make it impossible via types/fn signatures (e.g., MainEngine::new() detects root internally). (2) Runtime lint — wm_lint.check scans for violations (e.g., rogue .wm/ walkdir scan). (3) CI — fail the build. Each level catches what the previous misses.
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
