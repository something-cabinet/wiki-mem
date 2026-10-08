---
title: Self-install removed — MCP config always writes wm-cli
type: memory
tags: [deployment, npm, install, decision]
status: active
---

schema_version: 1
state: |-
  WM distribution decision (2026-07-31): self-install (~/.wm/bin + PATH via wm upgrade / wm init --full) is redundant with cargo-npm distribution and broken on macOS (ensure_on_path writes ~/.profile which zsh ignores). Removed entirely. MCP config generation (wm setup opencode) always writes "command": "wm-cli" — the user is assumed to have it on PATH. D1-D4 locked in @wiki/specs/remove-self-install-flow.
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
