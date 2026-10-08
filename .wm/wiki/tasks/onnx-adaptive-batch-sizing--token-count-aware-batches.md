---
id: wiki:tasks:onnx-adaptive-batch-sizing--token-count-aware-batches
title: ONNX Adaptive Batch Sizing — Token-count-aware batches
type: task
status: done
priority: medium
tags:
- from-spec
- spec:onnx-incremental-and-optimization
spec: specs/onnx-incremental-and-optimization
acceptance_criteria:
- text: 'AC-5: 10-token text uses larger batch than 500-token text'
---

schema_version: 1
state: |-
  id: wiki:tasks:onnx-adaptive-batch-sizing--token-count-aware-batches

  FR-5: Measure total token count per batch candidate, cap at 32,768 tokens. Short texts get larger batches, long texts get smaller batches.
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
