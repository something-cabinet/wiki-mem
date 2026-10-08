---
title: indicatif spinners require enable_steady_tick
type: memory
tags: [cli, rust, indicatif, failure]
status: active
---

schema_version: 1
state: |-
  indicatif spinner won't animate without enable_steady_tick(duration). Always call it after new_spinner() to start the background ticker. Full docs: @wiki/howto/indicatif-cli-progress, failure: @wiki/concepts/spinner-without-steady-tick
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
