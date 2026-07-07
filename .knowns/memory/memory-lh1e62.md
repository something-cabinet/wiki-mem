---
id: lh1e62
title: Pre-Release Crate API Drift Fix
layer: project
category: failure
tags:
  - onnx
  - embed
  - dependencies
createdAt: '2026-07-06T17:43:12.766Z'
updatedAt: '2026-07-06T17:43:12.766Z'
---

ort 2.0.0-rc.12 API drift fix: check actual src/ files (not docs), use (Vec,Vec) tuples for tensor creation instead of ndarray, build --features embed in CI. Full reference: @doc/learnings/learning-post-build-quality-pass-spec-alignment-tui-mcp-integration
