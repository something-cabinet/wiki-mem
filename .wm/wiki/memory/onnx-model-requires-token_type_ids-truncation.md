---
title: ONNX model requires token_type_ids + truncation
type: memory
tags: [onnx, embedding, model, failure]
status: active
---

schema_version: 1
state: |-
  bge-small-en-v1.5 ONNX model requires 3 inputs: input_ids, attention_mask, token_type_ids (zero for single-sentence). Tokenizer truncation must be set to 512 (BERT max positions). Without either, inference silently fails with opaque ONNX Runtime errors. Full reference: @wiki/concepts/onnx-token-type-ids-truncation
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
