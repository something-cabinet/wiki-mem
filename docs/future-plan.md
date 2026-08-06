# Future Plan — Investigating Concepts

**Reference:** Research-driven roadmap of concepts under investigation for wiki-mem. Derived from the DeltaNet paper (NeurIPS 2024) and WM's existing architecture.

## Overview

This document tracks concepts wiki-mem is investigating. Each entry records the motivating problem, the proposed direction, current status, and where it would land in the codebase. Statuses: `under investigation` → `proposed` → `in progress` → `done` / `rejected`.

## Concept Queue

| # | Concept | Status | Where it lands |
|---|---|---|---|
| 1 | Confidence-weighted retrieval (β analogue) | under investigation | `search.scoring`, RRF fusion |
| 2 | Supersedes-aware retrieval | under investigation | index build, search ranking |
| 3 | Associative recall evaluation | proposed | tests / benchmarks |
| 4 | Hybrid precision tiering | under investigation | search + ref resolution architecture |
| 5 | Memory write semantics (source concept) | captured | @wiki/concepts/memory-write-semantics |

## 1. Confidence-Weighted Retrieval (β Analogue)

**Motivation:** DeltaNet's data-dependent write strength β controls how firmly new information overwrites old. WM has page statuses (`draft`, `reviewed`, `approved`, `active`, ...) and a `confidence` frontmatter field, but neither influences ranking today — a draft note can outrank an accepted ADR on the same subject.

**Proposal:** Per-status authority multiplier in `search.scoring` (patterned after the existing `memory_salience_boost`), applied before RRF fusion. Reviewed/approved/active pages get a boost; draft/superseded pages get suppressed. The `confidence` field becomes a continuous per-page weight rather than inert metadata.

**Open questions:**
- What authority ladder maps statuses to multipliers?
- Should authority apply globally or only on near-ties (post-RRF)?

## 2. Supersedes-Aware Retrieval

**Motivation:** The delta rule's erase-before-write semantics — a superseded association must stop competing. `supersedes` exists as an edge type, but search does not read the graph, so superseded pages still surface at full weight.

**Proposal:** Deprioritize (or exclude) pages that are the target of an active `supersedes` edge at index time or rank time, so contradictory versions never surface at equal weight.

**Open questions:**
- Hard-exclude vs. rank penalty?
- How to handle superseded chains (A supersedes B supersedes C)?

## 3. Associative Recall Evaluation

**Motivation:** The paper's central finding — additive memories fail at *specific* recall, not general similarity. WM needs a benchmark on the same axis: given a task/query, retrieve the exact spec/decision document, not merely similar noise.

**Proposal:** Needle-in-haystack benchmark over the wiki: a curated set of (query → expected page ID) pairs, scored by rank and hit@k, run against hybrid, keyword, and semantic modes separately.

**Open questions:**
- Seed set from real task→spec resolutions?
- CI integration vs. manual run?

## 4. Hybrid Precision Tiering

**Motivation:** DeltaNet architecture: cheap linear layers everywhere, a few expensive global-attention layers where recall precision matters. WM analog: BM25 + embeddings for bulk queries, exact `@wiki/ref` resolution + graph traversal + LSP/code queries as the never-fuzzed precision tier.

**Proposal:** Formalize the tier boundaries; ensure exact-reference resolution is always attempted before semantic fallback on recall-critical paths; document when each tier is appropriate.

## Related

- @wiki/concepts/memory-write-semantics — source concept for items 1–4
- `docs/search-scoring-formula.md` — scoring pipeline that items 1–2 extend
- @wiki/concepts/edge-types — `supersedes` semantics (item 2)
- @wiki/concepts/cross-entity-search — fusion layer (items 1, 3)
