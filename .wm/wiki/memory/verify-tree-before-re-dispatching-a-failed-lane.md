---
title: Verify tree before re-dispatching a failed lane
type: memory
tags: [orchestration, subagents, workflow]
status: active
---

schema_version: 1
state: |-
  A failed/cancelled subagent lane often still wrote complete code to the tree. Before re-dispatching, run git status + check the expected artifacts + cargo check — if the work is there and compiles, reconcile instead of redoing (saves minutes-hours and avoids edit conflicts). Full: @wiki/patterns/verify-tree-before-redispatching-failed-lane
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
