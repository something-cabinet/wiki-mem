---
title: HTTP/WASM Architecture Cleanup — patterns and decisions
type: memory
tags: [architecture, angular, wasm, engine-port]
status: active
---

schema_version: 1
state: |-
  EnginePort: Introduced typed EnginePort injection token + HttpEngineService + MockEngineService. Angular services now return typed interfaces instead of `any`. All 6 consumer components migrated to @Inject(ENGINE_PORT).

  WASM crates (fjadra pattern): Created 3 new wasm-bindgen crates — graph-algo-wasm (petgraph BFS algorithms, ~142KB), bm25-rerank-wasm (~127KB), md-parse-wasm. All follow the fjadra pattern: fs-free, tokio-free, rayon-optional, wasm-pack build --target web, dynamic import in Angular.

  Layout cleanup: Removed dead HTTP /api/graph/layout endpoint + SSE stub + Angular computeLayout() + mock mappings.
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
