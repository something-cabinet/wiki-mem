---
title: dialoguer for Rust CLI prompts
type: memory
tags: [cli, rust, ux, pattern]
status: active
---

schema_version: 1
state: |-
  Use dialoguer for styled CLI prompts (Confirm, Select, MultiSelect). Arrow keys + space bar instead of bare stdin. Always guard with is_terminal() check and --no-wizard flag. Full docs: @wiki/howto/dialoguer-cli-prompts
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
