---
title: Drive the wm MCP over stdio when the live server is stale
type: memory
tags: [mcp, tool-reliability]
status: active
---

schema_version: 1
state: |-
  A long-lived MCP server keeps the binary it started with; 0.5.2 check_ac returned success without writing. Pipe JSON-RPC lines to a fresh wm-cli mcp and read the file back. Full reference: @wiki/howto/drive-the-wm-mcp-tools-over-stdio-from-a-script
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
answers:
  confidence: high
  dedup_action: new
  layer: project
  store_or_skip: true
