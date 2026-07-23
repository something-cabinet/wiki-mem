---
title: HTTP/WASM Architecture Cleanup — patterns and decisions
type: memory
tags: [architecture, angular, wasm, engine-port]
status: active
---

EnginePort: Introduced typed EnginePort injection token + HttpEngineService + MockEngineService. Angular services now return typed interfaces instead of `any`. All 6 consumer components migrated to @Inject(ENGINE_PORT).

WASM crates (fjadra pattern): Created 3 new wasm-bindgen crates — graph-algo-wasm (petgraph BFS algorithms, ~142KB), bm25-rerank-wasm (~127KB), md-parse-wasm. All follow the fjadra pattern: fs-free, tokio-free, rayon-optional, wasm-pack build --target web, dynamic import in Angular.

Layout cleanup: Removed dead HTTP /api/graph/layout endpoint + SSE stub + Angular computeLayout() + mock mappings.