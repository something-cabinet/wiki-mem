---
title: MCP validation serves stale graph until index rebuild
type: memory
tags: [validation, mcp, index-rebuild, staleness, failure]
status: active
---

schema_version: 1
state: |-
  After bulk direct-to-disk wiki edits (sed, parallel fixers, scripts), wm_validate.check via MCP serves the STALE in-memory graph snapshot — output is byte-identical to the pre-fix run until wm_index_rebuild (skip_embed=true) is called. A byte-identical validate result after edits = staleness signal, not "fixes didn't land". Use `wm-cli validate` (fresh in-process engine) as ground truth. Full reference: @wiki/concepts/failure-stale-mcp-validation-after-disk-edits
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
