---
title: 'Model-backed MCP tools: async + cached backend'
type: memory
tags: [mcp, async, caching, model-runtime]
status: active
---

schema_version: 1
state: |-
  Model-backed MCP tools must be async (register_typed_async + spawn_blocking), cache the backend once (keyed by dir+revision), stream-hash downloads, and error on a missing backend result instead of fabricating labels[0]. Applied to wm_decision (TD-13/14/15). Full: @doc/patterns/model-backed-mcp-tool-async-and-cached
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
