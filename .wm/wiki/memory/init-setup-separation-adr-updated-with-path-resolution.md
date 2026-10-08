---
title: init-setup-separation ADR updated with path resolution
type: memory
status: active
tags: [cli, init, setup, decision]
---

schema_version: 1
state: |-
  init-setup-separation ADR documented two-tier path resolution: init uses canonical wm-cli, setup resolves actual binary path. UPDATE (2026-07-31): self-install flow removed per @wiki/specs/remove-self-install-flow — wm init --full and wm upgrade no longer exist; MCP configs always write "wm-cli".
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
