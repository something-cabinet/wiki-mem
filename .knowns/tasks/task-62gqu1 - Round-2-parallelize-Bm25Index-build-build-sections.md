---
id: 62gqu1
title: 'Round 2: parallelize Bm25Index::build(), build_sections_from_wiki(), build_embeddings() phases'
status: done
priority: medium
labels:
  - rayon
  - parallelism
  - performance
createdAt: '2026-07-09T17:31:32.086Z'
updatedAt: '2026-07-09T17:38:13.157Z'
timeSpent: 0
---
# Round 2: parallelize Bm25Index::build(), build_sections_from_wiki(), build_embeddings() phases

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Structural changes: (1) Bm25Index::build() — split into parallel per-doc map + sequential merge, (2) build_sections_from_wiki() — collect paths then par_iter().map() read/parse, (3) build_embeddings() phases 1 & 3 — .par_iter() over sections.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

