---
title: Cross-entity wm_search.query with type + RRF + graph
type: task
status: done
tags: [from-spec, go-mode]
priority: high
id: jinq2x
acceptance_criteria:
  - text: "wm_search.query accepts a type param (default 'all') and merges results per entity type via RRF with a type field on each result"
  - text: "Graph propagation (depth 1, priority-weighted) is included as a third RRF input with recency boost via FSRS on tasks and salience boost on critical memory"
  - text: "Combined boosts are capped at 4x"
---

# Cross-entity wm_search.query with type + RRF + graph

> *Imported from Knowns task `jinq2x`*

# Cross-entity wm_search.query with type + RRF + graph

## Description


Extend wm_search.query: type param (default "all"), per-type RRF merge, recency boost via FSRS on tasks, salience boost on critical memory, graph propagation as third RRF input (depth 1, priority-weighted), boost cap at 4x, type field on results


## Acceptance Criteria
