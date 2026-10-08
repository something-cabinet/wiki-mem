---
title: All HTTP services must unwrap {success, data} envelope
type: memory
tags: [angular, http, api, consistency]
status: active
---

schema_version: 1
state: |-
  Inconsistent envelope handling between HttpEngineService and HttpCodeIntelService caused silent undefined data reads. Convention: extract {success, data}, throw on !success. Extract a shared httpCall helper. Full reference: @wiki/concepts/response-envelope-inconsistency
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
