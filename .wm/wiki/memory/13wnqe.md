---
title: Codex TOML vs JSON config format
type: memory
tags: [codex, config, toml]
created_at: "2026-07-06T18:50:55.788Z"
updated_at: "2026-07-06T18:50:55.788Z"
---

schema_version: 1
state: |-
  Codex uses `.codex/config.toml` with `[mcp_servers.wm]` TOML sections, NOT `.mcp.json` with JSON. The claude/codex combined arm was wrong because they diverge in format and path. Always research each platform's documented config format independently.
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
