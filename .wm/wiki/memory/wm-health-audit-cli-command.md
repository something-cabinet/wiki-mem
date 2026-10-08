---
title: wm health audit CLI command
type: memory
tags: [cli, health, audit]
status: active
---

schema_version: 1
state: |-
  `wm health audit` is a CLI command that scans wiki health. Default mode is dry-run. Use `--fix` to apply fixes. Flags: `--dry-run`, `--fix`, `--format json|text`. Detects empty pages (no parseable sections), broken relates_to refs (target pages that don't exist), and graph cycles. Fix mode: deletes stale empty task pages (0 inbound refs), case-corrects broken refs when the target exists with different casing, and removes truly broken refs.
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
