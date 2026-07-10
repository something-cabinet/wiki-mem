---
id: icdp50
title: 'Add rayon dep + Round 1: parallelize Bm25Index::search() and top_k_cosine()'
status: done
priority: high
labels:
  - rayon
  - parallelism
  - performance
createdAt: '2026-07-09T17:31:31.195Z'
updatedAt: '2026-07-09T17:38:12.848Z'
timeSpent: 0
---
# Add rayon dep + Round 1: parallelize Bm25Index::search() and top_k_cosine()

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add rayon = "1" to wm-core/Cargo.toml and wm-cli/Cargo.toml. Then make two one-liner changes: (1) Bm25Index::search() in search.rs — .par_iter().map() over docs, (2) top_k_cosine() in embed.rs — .par_iter().map() over vectors.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

