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

id: wiki:tasks:onnx-int8-quantization--model-quantization-for-cpu-speedup

FR-6: Convert the ONNX model to int8 with dynamic quantization for 2-3x CPU inference speedup.