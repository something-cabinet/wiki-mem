---
title: Run the CI check job locally before pushing a tag
type: memory
tags: [ci, release, gates]
status: active
---

schema_version: 1
state: |-
  Tests and clippy passing is not the CI check job: it also enforces a 300-line budget per wm-cli file, a 13000-line ceiling on wm-core/src, a dependency deny-list that counts dev-dependencies, and cargo audit. Run the gate scripts and cargo audit before any push or tag. Full reference: @wiki/howto/run-the-ci-check-job-locally-before-pushing
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
