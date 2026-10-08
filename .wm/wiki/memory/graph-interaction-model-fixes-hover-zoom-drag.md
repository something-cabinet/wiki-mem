---
title: Graph interaction model fixes — hover, zoom, drag
type: memory
tags: [graph, interaction, bug]
status: active
---

schema_version: 1
state: |-
  Three critical graph interaction bugs: (1) Split pointermove into two handlers — one unconditional for hover, one for drag/pan. (2) Zoom clamp floor of 0.1 blocked zoom when fitToView produces K=0.06 — lowered to 0.01. (3) Dragged node snapped back because pointerup set fx=x (pre-drag) instead of x=fx (post-drag). Always prefer fx/fy (pinned) over x/y in render, hit-test, and fitToView. Full reference: @doc/patterns/canvas2d-wasm-graph
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
