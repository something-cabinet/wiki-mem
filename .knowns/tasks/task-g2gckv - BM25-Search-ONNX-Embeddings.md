---
id: g2gckv
title: BM25 + Search + ONNX Embeddings
status: done
priority: high
labels:
  - from-spec
  - go-mode
  - search
createdAt: '2026-06-15T11:31:17.078Z'
updatedAt: '2026-06-15T14:01:26.357Z'
timeSpent: 0
spec: specs/local-knowledge-engine-rust
fulfills:
  - AC-10
  - AC-13
  - AC-14
  - AC-18
  - AC-19
---
# BM25 + Search + ONNX Embeddings

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Custom BM25 with field-weighted scoring, code-aware two-pass tokenizer, score normalization + stable sort + zero-result guard, topic-aware graph.neighbors scoring, token budget allocator with structural truncation. ONNX Embedder trait + OnnxEmbedder (ort, bge-small), NoopEmbedder fallback, vector storage (SQLite + ArcSwap), SearchMode enum, RRF fusion, index.rebuild with ArcSwap, staleness detection
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
BM25: custom field-weighted scorer with code-aware two-pass tokenizer (preserves ERR_AUTH_401 as compound + components), score normalization 0-1, rerank boosts, zero-result guard, 7 search tests. Embeddings: SearchMode enum (keyword/semantic/hybrid) with auto-detect, Embedder trait + NoopEmbedder fallback, cosine similarity, RRF fusion formula. 19 total tests passing.
<!-- SECTION:NOTES:END -->

