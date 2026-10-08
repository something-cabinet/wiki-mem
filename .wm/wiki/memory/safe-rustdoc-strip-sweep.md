---
title: Safe rustdoc-strip sweep
type: memory
tags: [comments, sweep, rust, schemas]
status: active
---

schema_version: 1
state: |-
  Ban-all-comments sweep: strip only lines whose first non-whitespace token is `///` or `//!` — never bare `//` (36 string/URL hazard lines like `"Read(//**)"`, `https://`). Preserve `#[schemars(description=...)]` (MCP schemas come from attributes, not `///`) and string literals. 1406 lines/113 files removed, zero warnings. Full: @doc/patterns/safe-rustdoc-strip-sweep
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
