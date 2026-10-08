---
title: Decision models cannot generalize across user wikis
type: memory
tags: [decision-models, generalization, product, gliner-rs]
status: active
---

schema_version: 1
state: |-
  Typed-decision models can't serve WM's product: zero-shot is too weak (loses to majority baseline on our own docs), fine-tuning only specializes to OUR domain (user wikis stay zero-shot + mismatched), and per-user training is infeasible (no labels, no versioning, users won't train). Keep heuristics + agent reasoning. Full: @doc/concepts/decision-models-cannot-generalize-across-user-wikis
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
