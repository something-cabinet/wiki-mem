---
title: Cross-entity hybrid search (pages + tasks + memory + docs)
type: task
status: done
tags: [memory, search, knowns, parity]
priority: high
id: 4hk4kz
spec: specs/cross-entity-hybrid-search
relates_to:
  - {type: implements, target: wiki:specs:cross-entity-hybrid-search}
acceptance_criteria:
  - text: "wm_search.query({q, type: 'all'}) returns pages and memory entries in a single ranked list, each result carrying a type field"
  - text: "wm_search.retrieve accepts a type param: memory context assembled as flat text, pages via graph BFS"
  - text: "Debounced IndexScheduler (500ms) coalesces rapid writes and wm_index.status reports per-type doc counts"
---

# Cross-entity hybrid search (pages + tasks + memory + docs)

> **Spec:** `specs/cross-entity-hybrid-search`

> *Imported from Knowns task `4hk4kz`*

# Cross-entity hybrid search (pages + tasks + memory + docs)

## Description


Match Knowns' `knowns search` and `knowns retrieve` capabilities: a single search call that queries across wiki pages, tasks, memory entries, and docs simultaneously, with hybrid RRF fusion (BM25 + semantic) and ranked results.

Current WM search only queries wiki pages. Tasks and memory are separate non-searchable lists.

What's needed:
1. **Unified SearchIndex** — BM25 index that spans pages + tasks + memory + docs, each tagged with their entity type
2. **wm_search.query `type` filter** — already exists for pages, extend to filter memory/tasks/docs or "all"
3. **wm_search.retrieve** — extend to assemble context from any entity type, not just pages
4. **Hybrid default** — RRF fusion on by default (BM25 + semantic embeddings) when embed feature is enabled
5. **Cross-entity results** — single ranked list mixing pages, tasks, memory snippets, and doc excerpts with type labels
6. **Tests** — integration tests for cross-entity search, hybrid ranking, result labeling


## Acceptance Criteria



## Implementation Notes


Spec approved. 8 decisions locked, 12 ACs, all Oracle findings resolved. ScoringConfig with 14 parameters in config.json search.scoring.
All 12 ACs implemented in this session. Cross-entity search, type filter, RRF merge, FSRS recency, salience boost, memory retrieve, per-type status, IndexScheduler.
