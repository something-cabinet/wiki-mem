---
title: Scope a commit by positive allow-list, never a deny-list
type: memory
tags: [workflow, git, orchestration, review]
status: active
---

schema_version: 1
state: |-
  Staging "the wiki records" by excluding known-foreign slugs produced an 88-file candidate set containing about fifty records belonging to other sessions (`gwl-*`, `cwr-*`, `curate-01`, `x-01`/`x-02`, `global-wiki-layer`, `configurable-wm-root`, `peer-landscape-2026`), all already uncommitted before the session began. A deny-list cannot enumerate what it has never seen.

  Use a positive allow-list: enumerate the exact paths you created, derive their side-artifacts (.wm page history lives in `.wm/versions/<kind>-wiki-<type>-<slug>.json`), then guard the stage with a regex that rejects anything outside the set.

  Print the staged list before committing when working in a shared tree, and confirm the staged diff shows what you expect (for example that a status flip really contains `+status: done`).

  Refs: @wiki/tasks/cli-golden-gate-silently-verifies-a-stale-binary-no-rebuild-step
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
