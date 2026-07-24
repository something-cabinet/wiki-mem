---
id: wiki:tasks:onnx-chunking-version-tracking--detect-chunking-logic-changes
title: ONNX Chunking Version Tracking — Detect chunking logic changes
type: task
status: todo
priority: medium
tags: [from-spec, spec:onnx-incremental-and-optimization]
spec: specs/onnx-incremental-and-optimization
acceptance_criteria:
  - text: "AC-9: After modifying section splitting logic, next rebuild regenerates all embeddings"
---
id: wiki:tasks:onnx-chunking-version-tracking--detect-chunking-logic-changes

FR-8: Hash the chunking logic's config/version string and store alongside section hashes. Mismatch triggers full re-embed.