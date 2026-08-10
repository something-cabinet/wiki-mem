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

id: wiki:tasks:onnx-adaptive-batch-sizing--token-count-aware-batches

FR-5: Measure total token count per batch candidate, cap at 32,768 tokens. Short texts get larger batches, long texts get smaller batches.