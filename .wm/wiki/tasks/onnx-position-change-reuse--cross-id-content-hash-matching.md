---
id: wiki:tasks:onnx-position-change-reuse--cross-id-content-hash-matching
title: ONNX Position-Change Reuse — Cross-ID content hash matching
type: task
status: done
priority: medium
tags:
- from-spec
- spec:onnx-incremental-and-optimization
spec: specs/onnx-incremental-and-optimization
acceptance_criteria:
- text: 'AC-3: Rename a page (same content, different path), rebuild — no re-embedding'
- text: 'AC-5: Cross-ID hash matching works'
---

schema_version: 1
state: |-
  id: wiki:tasks:onnx-position-change-reuse--cross-id-content-hash-matching

  FR-3: When a section's content hash exists in the store under a different section ID (page renamed), reuse the existing vector instead of re-embedding.
questions:
  - id: work_kind
    type: choice
    instructions: What kind of work is this task?
    options:
    - feature
    - bugfix
    - refactor
    - docs
    - test
    - chore
    - migration
  - id: priority
    type: choice
    instructions: What priority is this task?
    options:
    - low
    - medium
    - high
    - urgent
  - id: needs_spec
    type: noul
    instructions: This task depends on a spec.
  - id: has_ac
    type: noul
    instructions: This task has at least one acceptance criterion.
answers: {}
