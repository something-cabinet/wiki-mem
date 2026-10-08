---
title: Graph index staleness: write handlers need disk fallback
type: memory
tags: [tool-reliability, graph-index, pattern]
status: active
---

schema_version: 1
state: |-
  When write handlers (update/delete/task ops) resolve pages ONLY via the in-memory graph index, a stale index causes phantom "page not found" for pages that exist on disk — while get still works. Fix: graph-first/disk-fallback resolver (resolve_page_meta) wired into all page/task write handlers. Full reference: @wiki/patterns/stale-index-disk-fallback
questions:
  - id: layer
    type: choice
    instructions: Which memory layer does this entry belong to?
    options:
    - project
    - global
    - session
  - id: store_or_skip
    type: noul
    instructions: This entry is worth storing as durable memory.
  - id: dedup_action
    type: choice
    instructions: How should this entry relate to existing memory?
    options:
    - new
    - merge
    - supersede
    - skip
  - id: confidence
    type: score
    instructions: How confident is the recorded knowledge?
    levels:
    - low
    - medium
    - high
answers: {}
