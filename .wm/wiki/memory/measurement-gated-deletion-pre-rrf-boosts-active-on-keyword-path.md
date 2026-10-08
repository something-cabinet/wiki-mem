---
title: Measurement-gated deletion — pre-RRF boosts active on keyword path
type: memory
tags: [search, ranking, measurement]
status: active
---

schema_version: 1
state: |-
  Never delete scoring/ranking code without measuring recall@k per search path first. Pre-RRF boosts are inert for hybrid paths (rank-order only) but ACTIVE for keyword-only paths (score by value). A saturated golden eval (all multi-word queries) can miss exact-match regressions — add exact-title/id queries. Full reference: @wiki/patterns/measurement-gated-deletion
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
