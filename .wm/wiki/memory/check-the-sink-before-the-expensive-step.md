---
title: Check the sink before the expensive step
type: memory
tags: [persistence, tool-reliability]
status: active
---

schema_version: 1
state: |-
  index rebuild spent 294 CPU-seconds embedding into a store that could not save because vectors.db was locked, then reported success. Keep the open error, check persistability first, fail fast with the cause. Full reference: @wiki/patterns/check-the-sink-before-the-expensive-step
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
answers:
  confidence: high
  dedup_action: new
  layer: project
  store_or_skip: true
