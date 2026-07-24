---
id: wiki:tasks:onnx-model-version-tracking--detect-model-changes-and-trigger-full-re-embed
title: ONNX Model Version Tracking — Detect model changes and trigger full re-embed
type: task
status: todo
priority: medium
tags: [from-spec, spec:onnx-incremental-and-optimization]
spec: specs/onnx-incremental-and-optimization
acceptance_criteria:
  - text: "AC-2: Change the model file, rebuild — all vectors regenerated"
---
id: wiki:tasks:onnx-model-version-tracking--detect-model-changes-and-trigger-full-re-embed

FR-2: Store model_modified_at (file mtime) in hash cache. Compare on rebuild. Mismatch triggers full re-embed.