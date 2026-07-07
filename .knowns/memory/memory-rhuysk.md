---
id: rhuysk
title: Per-type BM25 + RRF + FSRS recency for cross-entity search
layer: project
category: pattern
tags:
  - search
  - ranking
  - fsrs
  - architecture
createdAt: '2026-07-07T05:08:49.338Z'
updatedAt: '2026-07-07T05:08:49.338Z'
---

WM's cross-entity search uses per-type BM25 indexes (pages + memory) merged via RRF. Tasks stay in page index with FSRS-6 recency boost. Memory gets flat text context in retrieve. recency_model config field with fsrs/linear/exponential/none options. Default stability 7 days. Full reference: @doc/specs/cross-entity-hybrid-search, @doc/handover/session-handover-cross-entity-search
