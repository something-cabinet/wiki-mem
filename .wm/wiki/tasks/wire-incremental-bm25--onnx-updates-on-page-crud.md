---
title: Wire incremental BM25 + ONNX updates on page CRUD
type: task
id: wiki:tasks:wire-incremental-bm25--onnx-updates-on-page-crud
status: done
priority: high
tags: [incremental, bm25, onnx, performance, ux]
implementation_notes: |-
  BM25 half was already DONE (update_bm25_for_page wired on create/update/delete).
  ONNX half wired 2026-08-07 (epic D4, task 4):
  - New `update_vectors_for_page(engine, page_id, sections, is_delete)` in
    apps/wm-core/src/page/services/page_crud_service.rs: removes every section
    vector belonging to the page from memory + turso
    (`VectorStore::remove_sections_for_page`), then — unless delete — embeds the
    freshly parsed sections and upserts them (`VectorStore::upsert_sections`).
    No full re-embed; only the affected page's sections are touched. No-ops on
    embed failures; if no embedder is loaded the stale vectors are still removed.
  - Wired into page create (page_crud_service.rs), page delete
    (page_crud_service.rs), and page update
    (page_update_builder_service.rs), mirroring each `update_bm25_for_page` call.
  - `wm_index.rebuild` still does full rebuild; version-tracking triggers
    (tasks #89/#74) handle model/chunking changes.
  Tests: `wm_embed::tests::test_upsert_sections_adds_to_store`,
  `test_remove_sections_for_page_clears_store`; wm-core e2e_pages/mcp_test suites
  green. Search now returns fresh results after page CRUD without a manual
  `wm_index_rebuild`.
  Known gap: `source_service.rs` (source complete/error) still only updates BM25,
  not ONNX — left out of this epic's scope; add
  `update_vectors_for_page` there when needed.
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
