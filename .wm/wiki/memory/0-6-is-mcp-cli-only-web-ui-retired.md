---
title: 0.6 is MCP + CLI only (web UI retired)
type: memory
tags: ['0.6', mcp-only, retirement, wm-server, wm-web]
status: active
---

schema_version: 1
state: |-
  0.6 ships MCP + CLI only: wm-web, wm-server, `wm web`, the HTTP MCP transport, and the browser-only WASM crates were deleted. MCP is in-process stdio (wm-cli depends only on wm-core). Nothing compiled against wm-server/wasm, so removal was a clean deletion. Full: @doc/decisions/retire-web-ui-ship-mcp-only-0-6
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
