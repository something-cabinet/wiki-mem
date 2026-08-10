---
id: wiki:tasks:onnx-cursor-scanning----since-flag-for-incremental-rebuild
title: ONNX Cursor Scanning — --since flag for incremental rebuild
type: task
status: done
priority: medium
tags:
- from-spec
- spec:onnx-incremental-and-optimization
spec: specs/onnx-incremental-and-optimization
acceptance_criteria:
- text: 'AC-8: --since 2026-07-01 only processes sections modified after that date'
---

id: wiki:tasks:onnx-cursor-scanning----since-flag-for-incremental-rebuild

FR-7: Add --since <timestamp> flag to wm index rebuild CLI command. Filter sections by page metadata updated_at.