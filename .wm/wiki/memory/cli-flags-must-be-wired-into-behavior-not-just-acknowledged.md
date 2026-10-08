---
title: CLI flags must be wired into behavior, not just acknowledged
type: memory
tags: [cli, ux, testing]
status: active
---

schema_version: 1
state: |-
  A clap flag that only logs "acknowledged" without threading the value into the underlying function is a silent no-op — users get a false sense of control (wm index code --skip-hash-check did nothing for 0.3.x). Wire the flag or remove it; add a test exercising the flag's path. Full ref: @wiki/concepts/inert-cli-flags-silent-noop
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
