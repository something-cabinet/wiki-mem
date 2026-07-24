---
id: wiki:tasks:onnx-deletion-reconciliation--remove-orphan-embeddings-on-rebuild
title: ONNX Deletion Reconciliation — Remove orphan embeddings on rebuild
type: task
status: todo
priority: medium
tags: [from-spec, spec:onnx-incremental-and-optimization]
spec: specs/onnx-incremental-and-optimization
acceptance_criteria:
  - text: "AC-1: Create a section, rebuild, delete the section, rebuild again — deleted section's vector is absent"
---
id: wiki:tasks:onnx-deletion-reconciliation--remove-orphan-embeddings-on-rebuild

FR-1: After embedding pass, query the vector store for IDs not in the current section list and delete them. Prevents ghost chunks from deleted pages appearing in search results.