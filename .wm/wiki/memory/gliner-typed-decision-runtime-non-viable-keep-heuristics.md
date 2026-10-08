---
title: gliner typed-decision runtime non-viable — keep heuristics
type: memory
tags: [gliner-rs, typed-decisions, eval, non-viable]
status: active
---

schema_version: 1
state: |-
  gliner local typed-decision runtime is NON-VIABLE as-is: best config (GLiNER2.5-Decide + key-facts + label-descriptions) = 0.491 vs 0.623 majority baseline (n=114). Beats the constant on choice (+.024) but loses on noul (−.238) and score (−.200); ECE .11–.32; 7–118 s/row CPU. Keep deterministic heuristics; feature off by default; fine-tune on WM labels is the only supported route. Do NOT use compact JSON (worse than prose). Full: @doc/decisions/gliner-decision-runtime-not-viable-keep-heuristics
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
