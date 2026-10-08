---
title: Curated clippy lint list + clippy.toml
type: memory
tags: [decision, clippy, lint]
status: active
---

schema_version: 1
state: |-
  Clippy uses curated lints in Cargo.toml `[workspace.lints.clippy]` + `clippy.toml` for config. No `all = warn`. Restriction-group lints (`as_conversions`, `cast_*`) cause worse code. Named `#[allow]` with reason permitted for correct code. Decision: @wiki/decisions/clippy-lint-curated-list-not-all
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
