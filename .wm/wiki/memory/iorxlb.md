---
title: Fantasy Benchmark — design target from user expectations
type: memory
tags: [strategy, benchmarking, product, critical]
created_at: "2026-07-10T08:57:37.451Z"
updated_at: "2026-07-10T08:57:37.451Z"
---

schema_version: 1
state: |-
  When auditing competitors, first-pass AI audits produce idealized fantasies (session memory, skill execution, tree-sitter) that are WRONG about the competitor but RIGHT about user expectations. Build toward the fantasy, not the reality. Use the gap between assumption and reality as your opportunity space. Full reference: @doc/learnings/the-fantasy-benchmark-compete-against-expectations-not-reality
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
