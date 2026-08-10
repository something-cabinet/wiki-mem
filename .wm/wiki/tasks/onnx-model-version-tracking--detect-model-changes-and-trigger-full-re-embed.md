---
id: wiki:tasks:onnx-model-version-tracking--detect-model-changes-and-trigger-full-re-embed
title: ONNX Model Version Tracking — Detect model changes and trigger full re-embed
type: task
status: done
priority: medium
tags: [from-spec, spec:onnx-incremental-and-optimization]
spec: specs/onnx-incremental-and-optimization
implementation_notes: |-
  Wired 2026-08-07 (epic D4 "minimal wired path"). The trigger mechanism already
  existed in `wm_embed::rebuild_embeddings_skip_unchanged` but every caller passed
  `EmbeddingMetadata::default()` + `model_path=None`, so the model_modified_at
  check never fired and the metadata was never persisted.
  What shipped:
  - `wm_embed::current_embedding_metadata(model_path)` computes
    `model_modified_at` from the ONNX file mtime + `chunking_version` from
    CARGO_PKG_VERSION (packages/wm-embed/src/lib.rs).
  - `rebuild_embeddings_skip_unchanged` now establishes a baseline on first run
    (when a model_path is provided but no metadata was ever persisted) by
    forcing a full re-embed — this is the one-time activation cost.
  - Metadata is persisted in a new turso `embed_meta` table
    (`VectorDb::store_metadata/load_metadata`, packages/wm-embed/src/vector_db.rs)
    and held on `VectorStore.embedding_metadata`
    (packages/wm-embed/src/services/vector_store_service.rs).
  - MCP `wm_index_rebuild` + `wm_index_embed` resolve the active model file via
    `$HOME/.wm/models/{model}/model.onnx` and thread `model_path` + persisted
    old metadata into the rebuild (apps/wm-core/src/mcp/tools/index.rs).
  Tests: `wm_embed::tests::test_model_change_triggers_full_reembed`,
  `test_metadata_persists_across_store_reload`. A model swap (mtime change) now
  forces regeneration of every vector (AC-2).
  Known gap: `wm-cli` (apps/wm-cli/src/main.rs:2868,2983) still passes
  `EmbeddingMetadata::default()` + `model_path=None` — the wm-core function keeps
  that path incremental (no baseline established) until the CLI lane threads
  metadata. MCP/daemon path is fully wired.
acceptance_criteria:
  - text: "AC-2: Change the model file, rebuild — all vectors regenerated"
---

id: wiki:tasks:onnx-model-version-tracking--detect-model-changes-and-trigger-full-re-embed

FR-2: Store model_modified_at (file mtime) in hash cache. Compare on rebuild. Mismatch triggers full re-embed.
