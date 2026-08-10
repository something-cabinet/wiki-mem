---
id: wiki:tasks:onnx-position-change-reuse--cross-id-content-hash-matching
title: ONNX Position-Change Reuse — Cross-ID content hash matching
type: task
status: done
priority: medium
tags:
- from-spec
- spec:onnx-incremental-and-optimization
spec: specs/onnx-incremental-and-optimization
acceptance_criteria:
- text: 'AC-3: Rename a page (same content, different path), rebuild — no re-embedding'
- text: 'AC-5: Cross-ID hash matching works'
---

id: wiki:tasks:onnx-position-change-reuse--cross-id-content-hash-matching

FR-3: When a section's content hash exists in the store under a different section ID (page renamed), reuse the existing vector instead of re-embedding.