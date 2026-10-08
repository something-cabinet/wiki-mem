---
id: wiki:tasks:onnx-int8-quantization--model-quantization-for-cpu-speedup
title: ONNX Int8 Quantization — Model quantization for CPU speedup
type: task
status: done
priority: medium
tags:
- from-spec
- spec:onnx-incremental-and-optimization
spec: specs/onnx-incremental-and-optimization
acceptance_criteria:
- text: 'AC-6: Quantized model loads and produces valid embeddings'
- text: 'AC-7: Cosine similarity fp32 vs int8 >= 0.99'
---

schema_version: 1
state: |-
  id: wiki:tasks:onnx-int8-quantization--model-quantization-for-cpu-speedup

  FR-6: Convert the ONNX model to int8 with dynamic quantization for 2-3x CPU inference speedup.
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
