---
id: wiki:memory:two-layer-regression-guards-lint-integration-tests
title: 'Two-layer regression guards: lint + integration tests'
type: memory
tags: [decision, testing, lint]
status: active
---

schema_version: 1
state: |-
  Use both lint checks (demand-driven, catches existing issues) AND integration tests (CI-driven, catches new regressions) for wiki health properties. Each layer has different trigger conditions and coverage. Full entry: @wiki/decisions/lint-plus-integration-tests-for-wiki-health
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
