---
title: Memory dedup threshold calibration + Bm25Index::search normalization gotcha
type: memory
tags: [memory-gate, bm25, calibration, threshold]
status: active
---

Heuristic memory write gate (spec @wiki/specs:memory-write-decision-gate). Raw Bm25Index::score_doc top-1 over this repo (3484 docs / 149 memory pages): exact-duplicate proxy min 156.89 (p10 251.9, p50 383.2), unrelated <=38, synthetic unrelated 3.6-7.5. DEFAULT_MEMORY_DEDUP_THRESHOLD = 100.0 chosen (below the exact-dup min, above every unrelated probe). GOTCHA: Bm25Index::search normalizes scores so top-1 == 1.0, making it unusable as an absolute skip threshold — use raw score_doc. D4 requires an eval step before lowering the threshold.