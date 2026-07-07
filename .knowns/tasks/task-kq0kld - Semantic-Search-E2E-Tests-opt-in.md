---
id: kq0kld
title: Semantic Search E2E Tests (opt-in)
status: done
priority: low
labels:
  - test
  - semantic
  - onnx
createdAt: '2026-07-06T17:40:26.971Z'
updatedAt: '2026-07-07T07:03:12.985Z'
timeSpent: 0
---
# Semantic Search E2E Tests (opt-in)

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Create wm-core/tests/semantic_test.rs behind #[cfg(feature = "embed")] gate: download model, index pages, test semantic search query returns results, test hybrid search RRF fusion, test model switch cleanup (AC-E19), test graceful degradation when model absent. Requires ONNX Runtime + model binary.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

