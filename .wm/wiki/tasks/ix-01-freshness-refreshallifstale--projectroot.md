---
title: 'IX-01 Freshness: refresh_all_if_stale + project_root'
type: task
id: "wiki:tasks:ix-01-freshness-refreshallifstale--projectroot"
status: done
priority: high
tags: [from-spec, spec:reinforce-indexing-flow, correctness, p0]
spec: specs/reinforce-indexing-flow
acceptance_criteria:
  - text: "refresh_all_if_stale used by search/validate/graph; stale_flag reflects all derived state"
  - text: "Index tools use engine.project_root"
  - text: "No stale reads in non-watcher contexts"
---

Freshness correctness: unified refresh_all_if_stale (graph+sections+BM25+optional embeddings) called from search/validate/graph; fix stale_flag semantics; use project_root in index tools. Per specs/reinforce-indexing-flow FR-1.