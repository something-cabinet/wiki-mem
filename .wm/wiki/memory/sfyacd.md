---
title: OpenCode MCP tool discovery via oh-my-opencode-slim plugin
type: memory
tags: [opencode, mcp, discovery, plugin]
created_at: "2026-07-09T07:54:46.243Z"
updated_at: "2026-07-09T07:54:46.243Z"
---

schema_version: 1
state: |-
  OpenCode surfaces MCP tools as agent functions based on oh-my-opencode-slim.json config. The orchestrator agent's mcps: ["*"] should auto-discover all MCP servers, but only works for fresh sessions. WM tools appear as wm_wm_* prefix (server name + tool prefix). Adding WM to both project opencode.json and ~/.config/opencode/opencode.json (global) ensures it's picked up. The tool response display may not work in some agent interfaces — use bash JSON-RPC as fallback.
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
