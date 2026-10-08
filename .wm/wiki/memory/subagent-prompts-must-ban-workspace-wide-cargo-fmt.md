---
title: Subagent prompts must ban workspace-wide cargo fmt
type: memory
tags: [workflow, subagents, cargo-fmt]
status: active
---

schema_version: 1
state: |-
  Subagent prompts for focused lanes must explicitly ban workspace-wide cargo fmt/fix. Always git diff --stat HEAD after subagent work lands to catch unexpected file-count inflation. Restore non-lane files to HEAD before committing. Full reference: @wiki/concepts/subagent-workspace-format-pollution
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
