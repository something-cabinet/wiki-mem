---
id: wiki:tasks:onnx-parallel-sessions--session-per-thread-for-concurrent-embedding
title: ONNX Parallel Sessions — Session-per-thread for concurrent embedding
type: task
status: todo
priority: medium
tags: [from-spec, spec:onnx-incremental-and-optimization]
spec: specs/onnx-incremental-and-optimization
acceptance_criteria:
  - text: "AC-4: Rebuild with 1,000 sections uses all CPU threads for ONNX inference"
  - text: "NFR-2: Memory stays under 2 GB total"
---
id: wiki:tasks:onnx-parallel-sessions--session-per-thread-for-concurrent-embedding

FR-4: Replace Mutex<Session> with thread-local ONNX sessions. Each rayon worker creates its own session for fully parallel embedding.