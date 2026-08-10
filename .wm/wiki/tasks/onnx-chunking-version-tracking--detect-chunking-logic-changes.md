---
id: wiki:tasks:onnx-chunking-version-tracking--detect-chunking-logic-changes
title: ONNX Chunking Version Tracking — Detect chunking logic changes
type: task
status: done
priority: medium
tags: [from-spec, spec:onnx-incremental-and-optimization]
spec: specs/onnx-incremental-and-optimization
implementation_notes: |-
  Wired 2026-08-07 (epic D4 "minimal wired path"). Same activation work as
  onnx-model-version-tracking: the chunking_version comparison existed but was
  dead because callers passed `EmbeddingMetadata::default()` and no metadata was
  ever persisted.
  What shipped:
  - `wm_embed::current_embedding_metadata` sets `chunking_version` from
    `env!("CARGO_PKG_VERSION")` (packages/wm-embed/src/lib.rs) — the crate
    version covers the section-splitting logic.
  - `rebuild_embeddings_skip_unchanged` forces a full re-embed when the persisted
    `chunking_version` differs from the current crate version, and establishes a
    baseline on the first run after wiring.
  - Metadata persisted in turso `embed_meta` table + `VectorStore` field; MCP
    index tools load the persisted baseline and persist the new one on save
    (packages/wm-embed/src/vector_db.rs,
    packages/wm-embed/src/services/vector_store_service.rs,
    apps/wm-core/src/mcp/tools/index.rs).
  Test: `wm_embed::tests::test_chunking_version_change_triggers_full_reembed`
  (a stale chunking_version forces regeneration of every vector, AC-9).
  Same known gap as #89: wm-cli passes default metadata (incremental preserved)
  until the CLI lane threads it.
acceptance_criteria:
  - text: "AC-9: After modifying section splitting logic, next rebuild regenerates all embeddings"
---

id: wiki:tasks:onnx-chunking-version-tracking--detect-chunking-logic-changes

FR-8: Hash the chunking logic's config/version string and store alongside section hashes. Mismatch triggers full re-embed.
