---
title: MCP tool field missing causes validation errors
type: memory
tags: [failure, mcp, task, validation]
status: active
---

schema_version: 1
state: |-
  When adding a new field to a page model, add it to ALL tool handlers (Create AND Update) or validation will break. wm_task.create was missing acceptance_criteria causing 153 false validation errors. Full reference: @wiki/concepts/failure-mcp-task-missing-acceptance-criteria
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
