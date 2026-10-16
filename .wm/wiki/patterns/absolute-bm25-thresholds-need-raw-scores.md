---
title: 'Pattern: Absolute BM25 thresholds need raw scores, not search()'
type: pattern
id: "wiki:patterns:absolute-bm25-thresholds-need-raw-scores"
status: draft
tags: [pattern, bm25, search, thresholds]
relates_to:
  - {type: references, target: wiki:tasks:implement-memory-write-decision-gate--heuristic-answering-of-the-memory-question-set}
---

## Problem
`Bm25Index::search` normalizes its scores so the top-1 result is always 1.0. Any decision gated on an ABSOLUTE score magnitude ("is this a near-duplicate?") silently always trips, because the compared value is normalized, not the raw weight.

## Solution
When an absolute threshold is required, score each candidate with the raw per-document API: `Bm25Index::score_doc(doc, &tokens)` over `index.docs` (filtering to the target corpus, e.g. ids prefixed `wiki:memory:`). Keep `Bm25Index::search` for ranked retrieval where only relative order matters.

## When to Use
Near-duplicate detection, confidence bands, or any cutoff expressed as a BM25 magnitude.

## When Not to Use
Top-k ranked retrieval where only ordering matters — normalized scores are correct there.

## Related
@wiki/tasks/implement-memory-write-decision-gate--heuristic-answering-of-the-memory-question-set
Calibration numbers: raw top-1 exact-duplicate min 156.89 vs unrelated <=38 over this repo's corpus.