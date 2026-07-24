---
title: ONNX model requires token_type_ids + truncation
type: memory
tags: [onnx, embedding, model, failure]
status: active
---

bge-small-en-v1.5 ONNX model requires 3 inputs: input_ids, attention_mask, token_type_ids (zero for single-sentence). Tokenizer truncation must be set to 512 (BERT max positions). Without either, inference silently fails with opaque ONNX Runtime errors. Full reference: @wiki/concepts/onnx-token-type-ids-truncation