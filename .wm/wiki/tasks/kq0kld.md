---
title: Semantic Search E2E Tests (opt-in)
type: task
status: done
tags: [test, semantic, onnx]
priority: low
id: kq0kld
acceptance_criteria:
  - text: "wm-core/tests/semantic_test.rs exists behind the #[cfg(feature = \"embed\")] gate"
  - text: "Semantic search query returns results and hybrid search RRF fusion is tested (including AC-E19 model switch cleanup)"
  - text: "Graceful degradation is verified when the model is absent"
---

# Semantic Search E2E Tests (opt-in)

> *Imported from Knowns task `kq0kld`*

# Semantic Search E2E Tests (opt-in)

## Description


Create wm-core/tests/semantic_test.rs behind #[cfg(feature = "embed")] gate: download model, index pages, test semantic search query returns results, test hybrid search RRF fusion, test model switch cleanup (AC-E19), test graceful degradation when model absent. Requires ONNX Runtime + model binary.


## Acceptance Criteria
