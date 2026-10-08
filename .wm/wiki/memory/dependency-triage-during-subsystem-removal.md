---
title: Dependency triage during subsystem removal
type: memory
tags: [pattern, cleanup, dependencies]
status: active
---

schema_version: 1
state: |-
  When removing a subsystem, triage each dependency by actual usage: subsystem-only deps get removed, deps shared with surviving features stay. Grep the source for call sites, not just Cargo.toml. Full reference: @wiki/patterns/dependency-triage-during-subsystem-removal
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
