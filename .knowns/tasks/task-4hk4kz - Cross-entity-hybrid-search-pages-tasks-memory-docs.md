---
id: 4hk4kz
title: Cross-entity hybrid search (pages + tasks + memory + docs)
status: done
priority: high
labels:
  - memory
  - search
  - knowns
  - parity
createdAt: '2026-07-07T03:59:23.781Z'
updatedAt: '2026-07-07T06:19:51.524Z'
timeSpent: 0
spec: specs/cross-entity-hybrid-search
---
# Cross-entity hybrid search (pages + tasks + memory + docs)

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Match Knowns' `knowns search` and `knowns retrieve` capabilities: a single search call that queries across wiki pages, tasks, memory entries, and docs simultaneously, with hybrid RRF fusion (BM25 + semantic) and ranked results.

Current WM search only queries wiki pages. Tasks and memory are separate non-searchable lists.

What's needed:
1. **Unified SearchIndex** — BM25 index that spans pages + tasks + memory + docs, each tagged with their entity type
2. **wm_search.query `type` filter** — already exists for pages, extend to filter memory/tasks/docs or "all"
3. **wm_search.retrieve** — extend to assemble context from any entity type, not just pages
4. **Hybrid default** — RRF fusion on by default (BM25 + semantic embeddings) when embed feature is enabled
5. **Cross-entity results** — single ranked list mixing pages, tasks, memory snippets, and doc excerpts with type labels
6. **Tests** — integration tests for cross-entity search, hybrid ranking, result labeling
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Spec approved. 8 decisions locked, 12 ACs, all Oracle findings resolved. ScoringConfig with 14 parameters in config.json search.scoring.
All 12 ACs implemented in this session. Cross-entity search, type filter, RRF merge, FSRS recency, salience boost, memory retrieve, per-type status, IndexScheduler.
<!-- SECTION:NOTES:END -->

