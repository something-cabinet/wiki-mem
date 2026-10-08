---
title: MCP proxy architecture — privileged /api/mcp channel + token split
type: memory
tags: [mcp, proxy, architecture, security]
status: active
---

schema_version: 1
state: |-
  MCP is a thin stdio→HTTP proxy to the wm-server daemon, targeting a privileged POST /api/mcp/tools/{list,call} channel with a SEPARATE mcp-token (web-token stays read-only). Dynamic tools/list from the registry — no STATIC_TOOLS array. Full: @wiki/decisions/mcp-proxy-privileged-channel-token-split
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
