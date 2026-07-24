---
id: wiki:specs:onnx-incremental-and-optimization
title: ONNX Embedding Pipeline — Incremental Sync + Performance Optimization
type: spec
status: approved
tags: [spec, onnx, embedding, search, performance, approved]
---
id: wiki:specs:onnx-incremental-and-optimization

## Overview

Harden the ONNX embedding pipeline across three dimensions: correctness (deletion handling, model version tracking, position-change reuse), performance (int8 quantization, session-per-thread parallelism, adaptive batching), and maintainability (cursor-based scanning, chunking version tracking). Building on the existing `rebuild_embeddings_skip_unchanged` content-hash foundation.

Reference: @wiki/specs/onnx-embedding-integration (existing ONNX integration), @wiki/reference:search-scoring-formula, @wiki/concepts:bm25-search

## Locked Decisions

- D1: All ONNX gaps in scope — deletion reconciliation, model version tracking, position-change reuse, dual-index (named vectors), session-per-thread parallelism, adaptive batch sizing, int8 quantization
- D2: Both CLI (single-user) and server (multi-user) deployment profiles
- D3: Named vectors approach for model migration — add new vector dimension to existing collection, background update, switch query `using` param, delete old vector
- D4: Session-per-thread — each rayon worker creates its own ONNX session for fully parallel embedding. Acceptable memory overhead (~200MB × thread count)
- D5: Content-hash cross-ID matching — when a section's content is unchanged but its ID changed (page renamed), reuse the old vector instead of re-embedding
- D6: Int8 dynamic quantization for 2-3x CPU inference speedup

## Requirements

### Functional Requirements

- FR-1: Deleted wiki pages/sections must have their embeddings removed from the vector store within one rebuild cycle
- FR-2: Model version must be tracked per vector. When the ONNX model file changes (new download or update), all embeddings must be regenerated on the next rebuild
- FR-3: When a section's content hash exists in the store under a different section ID (page renamed), the existing vector must be reused rather than re-embedded
- FR-4: Each rayon worker thread must have its own ONNX session to enable parallel embedding during rebuild
- FR-5: Batch size must adapt to input text length — shorter texts use larger batches, longer texts use smaller batches
- FR-6: The ONNX model must be quantized to int8 with dynamic quantization for CPU inference speedup
- FR-7: Index rebuild must support a `--since <timestamp>` flag to only scan sections modified after the given timestamp
- FR-8: Chunking strategy version must be stored in the hash cache. When the chunking logic changes, all sections must be re-embedded

### Non-Functional Requirements

- NFR-1: `wm index rebuild` must complete within 30 seconds for 2,000 sections (target: ~10s with parallel + quantized)
- NFR-2: Memory overhead from session-per-thread must not exceed 2 GB total
- NFR-3: Parallel embedding must not degrade search quality — vectors must be identical regardless of which thread produced them
- NFR-4: Quantized model must produce embeddings within 1% cosine similarity of the fp32 model on the same inputs

## Acceptance Criteria

- [ ] AC-1: Create a section, rebuild index, delete the section, rebuild again — deleted section's vector is absent from the store
- [ ] AC-2: Change the model file (touch/symlink a different one), rebuild — all vectors are regenerated
- [ ] AC-3: Rename a page (same content, different path), rebuild — no re-embedding occurs, old vector is reused
- [ ] AC-4: `wm index rebuild` with 1,000 sections uses all available CPU threads for ONNX inference (not serialized behind a mutex)
- [ ] AC-5: Embedding a 10-token text uses a larger batch than embedding a 500-token text
- [ ] AC-6: Quantized model loads and produces valid embeddings
- [ ] AC-7: Cosine similarity between fp32 and int8 embeddings of the same text is >= 0.99
- [ ] AC-8: `wm index rebuild --since 2026-07-01` only processes sections modified after that date
- [ ] AC-9: After modifying the section splitting logic, the next rebuild regenerates all embeddings (chunking version mismatch detected)

## Scenarios

### Scenario 1: Page Deletion Cleanup
**Given** a wiki with 100 sections and their embeddings in the vector store
**When** a page with 5 sections is deleted and `wm index rebuild` runs
**Then** the 5 orphan embeddings are removed from the store
**And** the remaining 95 embeddings are unchanged (no re-embedding)

### Scenario 2: Model Upgrade
**Given** the current model is `bge-small-en-v1.5` with 1,000 stored embeddings
**When** a new model version is downloaded and `wm index rebuild` runs
**Then** the hash cache detects the model version mismatch
**And** all 1,000 sections are re-embedded with the new model
**And** the vector store is updated atomically

### Scenario 3: Concurrent Rebuild
**Given** a wiki with 2,000 sections
**When** `wm index rebuild` runs on an 8-core machine
**Then** all sections are hashed in parallel (unchanged behavior)
**And** changed sections are embedded using all 8 cores simultaneously
**And** elapsed wall time is lower than with a single session

### Scenario 4: Quantized Inference
**Given** the ONNX model is quantized to int8
**When** `wm index rebuild` embeds 100 sections
**Then** the quantized model produces vectors within 1% cosine similarity of the fp32 model
**And** total inference time is reduced by at least 40% vs fp32

## Technical Notes

### Implementation Order

1. **Deletion reconciliation** — simplest, biggest correctness win. Query vector store, diff against current section IDs, delete orphans.
2. **Model version tracking** — store `model_modified_at` (file mtime) in hash cache. Compare on rebuild.
3. **Position-change reuse** — when computing changed sections, also check if unchanged content exists under a different ID.
4. **Parallel sessions** — replace `Mutex<Session>` with thread-local session creation via rayon's thread pool.
5. **Adaptive batch sizing** — measure total token count per batch candidate, cap at 32,768 tokens.
6. **Int8 quantization** — convert model via Optimum or ONNX Runtime quantization API.
7. **Cursor scanning** — add `--since` flag to `wm index rebuild` CLI command.
8. **Chunking version** — hash the chunking config, store alongside section hashes.

### Parallel Session Design

```rust
// Replace Mutex<Session> with thread-local:
thread_local! {
    static ONNX_SESSION: RefCell<Option<ort::session::Session>> = RefCell::new(None);
}
```

### Quantization Strategy

```bash
optimum-cli export onnx --model BAAI/bge-small-en-v1.5 --quantize int8 bge-small-int8/
```

## Open Questions

- [ ] Should quantization happen at download time or be pre-packaged?
- [ ] Memory per ONNX session? Need measurement before setting thread limits.
- [ ] Do all operators support int8 quantization in this model?