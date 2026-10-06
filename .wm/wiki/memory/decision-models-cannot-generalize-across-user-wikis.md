---
title: Decision models cannot generalize across user wikis
type: memory
tags: [decision-models, generalization, product, gliner-rs]
status: active
---

Typed-decision models can't serve WM's product: zero-shot is too weak (loses to majority baseline on our own docs), fine-tuning only specializes to OUR domain (user wikis stay zero-shot + mismatched), and per-user training is infeasible (no labels, no versioning, users won't train). Keep heuristics + agent reasoning. Full: @doc/concepts/decision-models-cannot-generalize-across-user-wikis