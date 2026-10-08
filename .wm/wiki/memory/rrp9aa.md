---
title: tools.rs → domain modules pattern
type: memory
tags: [architecture, refactor, mcp, tools]
created_at: "2026-07-07T10:34:46.491Z"
updated_at: "2026-07-07T10:34:46.491Z"
---

schema_version: 1
state: |-
  When a single MCP handler file grows beyond 1000 lines, split it into per-domain modules under mcp/tools/. Each domain module has a `pub fn register(registry, engine)` function. The parent tools.rs becomes a ~30-line delegator calling each module. Domain names match tool prefixes (search.rs → wm_search.*, page.rs → wm_page.*). This keeps handler code discoverable and prevents merge conflicts.
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
