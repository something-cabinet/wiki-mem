---
title: Canvas 2D + WASM graph rendering pattern
type: memory
tags: [graph, canvas, wasm, layout]
status: active
---

schema_version: 1
state: |-
  Graph uses Canvas 2D rendering (not WebGL/regl) with fjadra WASM for force-directed layout in the browser. Edges support bezier curves for bidirectional pairs, triangle arrowheads, and HTML overlay labels. Layout runs in WASM via fjadra. Full reference: @doc/patterns/canvas2d-wasm-graph
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
