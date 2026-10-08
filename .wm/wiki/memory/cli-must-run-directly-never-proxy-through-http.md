---
title: CLI must run directly, never proxy through HTTP
type: memory
tags: [cli, architecture]
status: active
---

schema_version: 1
state: |-
  wm-cli commands must execute in-process via create_engine(), never proxy through HTTP to wm-server. CLI tests, offline operation, and latency depend on this. Full reference: @wiki/decisions/cli-direct-execution-not-http-proxy
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
