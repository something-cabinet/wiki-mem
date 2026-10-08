---
title: When zero-shot decision models work
type: memory
tags: [zero-shot, decision-models, classification, eval]
status: active
---

Zero-shot decision models: local zero-shot works only for well-posed single-label distinct-label prose (GLiNER2 ~0.72, GLiClass ~0.70-0.74); task kind dominates (sentiment ~0.9, topic ~0.4-0.55, subjective/emotion ~0.25-0.35). Fails on structured typed decisions (base local models at/below majority) and high-cardinality labels. Fixes: a decision specialist (Julia-1 73%), fine-tuning (Laya .36->.77), or retrieval + per-candidate boolean + local reranker. Running on .wm data is still zero-shot (input != training). Full: @doc/patterns/when-zero-shot-decision-models-work