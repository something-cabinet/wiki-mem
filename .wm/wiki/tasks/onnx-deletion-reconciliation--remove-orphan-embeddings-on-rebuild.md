---
id: wiki:tasks:onnx-deletion-reconciliation--remove-orphan-embeddings-on-rebuild
title: ONNX Deletion Reconciliation — Remove orphan embeddings on rebuild
type: task
status: done
priority: medium
tags: [from-spec, spec:onnx-incremental-and-optimization]
spec: specs/onnx-incremental-and-optimization
implementation_notes: |-
  Wired 2026-08-07 (epic D4 "minimal wired path"). Orphan cleanup previously
  existed only inside `VectorDb::rebuild` (used by tests); the production
  rebuild path (`rebuild_embeddings_skip_unchanged` → `replace_entries_and_hashes`
  → `VectorStore::save_to_disk` → `store_vectors_raw`) was upsert-only, so
  deleted pages' vectors persisted in turso and still returned in search.
  What shipped:
  - New `VectorDb::store_vectors_sync` (packages/wm-embed/src/vector_db.rs):
    upserts the full in-memory map, then deletes every `chunks`/`content_hashes`
    row whose id is no longer present (orphan reconciliation).
  - `VectorStore::save_to_disk` now uses `store_vectors_sync` (upsert-only
    `store_vectors_raw` is kept for migration + incremental upserts)
    (packages/wm-embed/src/services/vector_store_service.rs).
  - `wm_index_rebuild` / `wm_index_embed` persist through `save_to_disk`, so the
    production rebuild now leaves zero orphan vectors.
  - Bonus: `VectorStore::remove_sections_for_page` + `VectorDb::delete_vectors_with_prefix`
    invalidate a single page's vectors on page delete/update (task
    wire-incremental-bm25--onnx-updates-on-page-crud).
  Tests: `wm_embed::vector_db::tests::test_sync_removes_orphan_vectors_on_rebuild`
  (embed a page, delete a section, sync — the deleted section's vector is gone
  from store and search, AC-1), plus
  `wm_embed::tests::test_remove_sections_for_page_clears_store`.
acceptance_criteria:
  - text: "AC-1: Create a section, rebuild, delete the section, rebuild again — deleted section's vector is absent"
---

id: wiki:tasks:onnx-deletion-reconciliation--remove-orphan-embeddings-on-rebuild

FR-1: After embedding pass, query the vector store for IDs not in the current section list and delete them. Prevents ghost chunks from deleted pages appearing in search results.
