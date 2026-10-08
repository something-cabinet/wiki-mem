---
title: Review gates: empty result = not reviewed
type: memory
tags: [review, orchestration, subagents]
status: active
---

schema_version: 1
state: |-
  A review-gate subagent that returns EMPTY is NOT a review — "completed" lifecycle only means session ended. Validate gate output before advancing a wave: empty/no-verdicts → treat as not-reviewed and re-dispatch with explicit per-lane GO/GO-with-findings/NO-GO requirement. Prefer per-lane gates over one mega-gate. NOTE 2026-08-14: ora-1 (ses_0059325aaffeAEG7e0RabO2xvp) produced 2 consecutive empty results on the same Wave-1 gate task — a dead delivery channel. Do not reissue to that session; spawn a FRESH oracle for gates. See wiki:concepts:empty-review-gate-result.
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
