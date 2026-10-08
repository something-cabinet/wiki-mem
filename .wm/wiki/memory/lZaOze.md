---
title: Single entry point — wm-cli is the only binary
type: memory
tags: [architecture, mcp, entry-point]
created_at: "2026-07-14T04:41:47.491Z"
updated_at: "2026-07-14T04:41:47.491Z"
---

schema_version: 1
state: |-
  wm-cli is the only standalone binary. wm-server, wm-vectors-bin are library crates. wm-cli mcp embeds the HTTP server in-process on a random port. wm-cli web embeds it on a user-specified port. No separate wm-mcp binary. Full reference: @doc/learnings/proxy-architecture-single-entrypoint
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
