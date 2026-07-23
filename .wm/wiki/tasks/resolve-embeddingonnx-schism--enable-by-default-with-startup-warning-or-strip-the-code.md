---
title: Resolve embedding/ONNX schism — enable by default with startup warning or strip the code
type: task
status: todo
priority: high
tags: [architecture, search, onnx, embedding]
---

From @oracle review S-3: wm-embed has full ONNX pipeline (~1400 lines) but all behind #[cfg(feature = "onnx")] which is NOT default. Without it, MainEngine gets NoopEmbedder — semantic search silently degrades to keyword-only BM25, RRF/vector infra is untestable. Either compile it in with a runtime toggle + loud startup warning, or delete it until enabled.