---
title: relates_to slash→colon normalization fix
type: memory
tags: [graph, relates_to, id, bug]
---

schema_version: 1
state: |-
  relates_to targets in frontmatter use inconsistent ID schemes — half use `/` separators (`wiki:specs/graph-and-ui-fix`) while node IDs use `:` separators (`wiki:specs:graph-and-ui-fix`). The `id_index` lookup silently dropped ~53% of entries (~26 edges).

  **Fix:** Added `target.replace('/', ':')` normalization before lookup at `apps/wm-core/src/graph/mod.rs:130`. Also added `tracing::debug!` log for still-unresolved targets.

  **Commit:** `fb28a96`
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
