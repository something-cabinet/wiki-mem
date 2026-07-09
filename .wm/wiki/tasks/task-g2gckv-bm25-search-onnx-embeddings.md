---
title: BM25 + Search + ONNX Embeddings
type: task
status: done
tags: [from-spec, go-mode, search]
priority: high
knowns_id: g2gckv
spec: specs/local-knowledge-engine-rust
fulfills: [AC-10, AC-13, AC-14, AC-18, AC-19]
---

# BM25 + Search + ONNX Embeddings

> **Spec:** `specs/local-knowledge-engine-rust`

> **Fulfills:** AC-10, AC-13, AC-14, AC-18, AC-19

> *Imported from Knowns task `g2gckv`*

# BM25 + Search + ONNX Embeddings

## Description


Custom BM25 with field-weighted scoring, code-aware two-pass tokenizer, score normalization + stable sort + zero-result guard, topic-aware graph.neighbors scoring, token budget allocator with structural truncation. ONNX Embedder trait + OnnxEmbedder (ort, bge-small), NoopEmbedder fallback, vector storage (SQLite + ArcSwap), SearchMode enum, RRF fusion, index.rebuild with ArcSwap, staleness detection


## Acceptance Criteria



## Implementation Notes


BM25: custom field-weighted scorer with code-aware two-pass tokenizer (preserves ERR_AUTH_401 as compound + components), score normalization 0-1, rerank boosts, zero-result guard, 7 search tests. Embeddings: SearchMode enum (keyword/semantic/hybrid) with auto-detect, Embedder trait + NoopEmbedder fallback, cosine similarity, RRF fusion formula. 19 total tests passing.
