---
title: gliner typed-decision runtime non-viable — keep heuristics
type: memory
tags: [gliner-rs, typed-decisions, eval, non-viable]
status: active
---

gliner local typed-decision runtime is NON-VIABLE as-is: best config (GLiNER2.5-Decide + key-facts + label-descriptions) = 0.491 vs 0.623 majority baseline (n=114). Beats the constant on choice (+.024) but loses on noul (−.238) and score (−.200); ECE .11–.32; 7–118 s/row CPU. Keep deterministic heuristics; feature off by default; fine-tune on WM labels is the only supported route. Do NOT use compact JSON (worse than prose). Full: @doc/decisions/gliner-decision-runtime-not-viable-keep-heuristics