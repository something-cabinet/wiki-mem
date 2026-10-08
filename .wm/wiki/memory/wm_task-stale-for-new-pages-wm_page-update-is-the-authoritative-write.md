---
title: wm_task stale for new pages — wm_page.update is the authoritative write
type: memory
status: active
tags: [tool-reliability, mcp, graph-index, fixed]
---

schema_version: 1
state: |-
  Phantom "page not found" on wm_page.update / wm_task.update for pages that exist on disk = stale in-memory graph index (write paths had no disk fallback while get did). FIXED 2026-08-07 via shared graph-first/disk-fallback resolver (resolve_page_meta in page_crud_service.rs); no workaround needed anymore. Full reference: @wiki/patterns/stale-index-disk-fallback
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
