---
id: n7a44o
title: Semantic search for memory entries via ONNX
status: done
priority: medium
labels:
  - sprint-2
  - feature
  - search
  - memory
createdAt: '2026-07-10T10:15:46.457Z'
updatedAt: '2026-07-10T11:24:36.765Z'
timeSpent: 356
assignee: '@me'
spec: specs/wm-leapfrog-replace-knowns-with-complete-memory-layer
fulfills:
  - AC-10
---
# Semantic search for memory entries via ONNX

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Embed memory entries at write time via ONNX embedder. Add vector search path for memory in wm_search.query with type: memory. Store vectors in existing vectors.bin format or separate memory vectors file.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented semantic search for memory entries:
- Added memory_vectors: ArcSwap<HashMap<String, EmbedVector>> to EngineState
- Memory entries are embedded at write time (wm_memory.add) when embedder is loaded
- Query supports Semantic and Hybrid modes for memory via cosine similarity on memory_vectors
- Added embed_memory_entry() helper in memory.rs
- Pre-existing bug fix: promote handler used undefined `path` variable (changed to `global_path`)
- 154 tests pass (previously 154, no regressions)
<!-- SECTION:NOTES:END -->

