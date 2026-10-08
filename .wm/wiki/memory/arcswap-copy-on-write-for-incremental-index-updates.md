---
title: ArcSwap copy-on-write for incremental index updates
type: memory
tags: [rust, graph, architecture]
status: active
---

schema_version: 1
state: |-
  For in-memory indices using ArcSwap, use copy-on-write for single-element mutations: load Arc, clone inner data, mutate clone, store new Arc. No reader blocking. Full reference: @wiki/patterns/arcswap-copy-on-write-incremental
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
