---
title: Wire incremental BM25 + ONNX updates on page CRUD
type: task
id: wiki:tasks:wire-incremental-bm25--onnx-updates-on-page-crud
status: todo
priority: high
tags: [incremental, bm25, onnx, performance, ux]
acceptance_criteria:
  - text: "Page create updates BM25 index incrementally (no full rebuild)"
  - text: "Page update updates BM25 incrementally (remove old sections, add new)"
  - text: "Page delete removes sections from BM25 incrementally"
  - text: "Page create/update/delete updates ONNX embeddings incrementally (re-embed only changed sections)"
  - text: "wm_index.rebuild still works for full rebuild when needed"
  - text: "Search returns fresh results without requiring manual wm_index.rebuild"
---

After every page create/update/delete, incrementally update BM25 and ONNX indexes instead of requiring a full `wm_index.rebuild`. The stale_flag is already set on mutations — the gap is the incremental wiring.

BM25: `add_document()` and `remove_document()` already implemented in `Bm25Index` at `packages/wm-search/src/services/bm25_index_service.rs`. Need to call these from page CRUD handlers.

ONNX: Need to compute embeddings for just the changed page's sections and update the vector store incrementally.

Pieces already exist:
- P5b (wiki:tasks:7d3aa1): BM25 incremental API — DONE (methods exist)
- P5c (wiki:tasks:b6d2ca): Single-file section parsing — needed for incremental
- Spec: onnx-incremental-and-optimization (wiki:specs:onnx-incremental-and-optimization) — covers ONNX side
- P5a (wiki:tasks:57bca4): File watcher for auto-detection

Integration points:
- page_crud_service.rs: create/delete handlers → update BM25 + ONNX
- page_update_builder_service.rs: update handler → update BM25 + ONNX  
- source_service.rs: source complete/error → update BM25 + ONNX