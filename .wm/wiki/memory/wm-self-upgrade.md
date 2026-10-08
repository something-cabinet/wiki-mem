---
title: wm self upgrade
id: wiki:memory:wm-self-upgrade
type: memory
tags: [deployment, npm, removal]
---

schema_version: 1
state: |-
  REMOVED (2026-07-31) per @wiki/specs/remove-self-install-flow: wm upgrade, wm init --full, and the wm_core::install module (~/.wm/bin copy + PATH registration) no longer exist. Distribution is npm (cargo-npm @something-cabinet/wm-cli) or cargo install. MCP configs always write "wm-cli". Legacy ~/.wm/bin folders are left in place (manual removal).
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
