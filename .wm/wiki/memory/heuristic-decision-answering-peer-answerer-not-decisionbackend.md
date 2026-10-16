---
title: 'Heuristic decision answering: peer answerer, not DecisionBackend'
type: memory
tags: [typed-decisions, heuristics, architecture]
status: active
---

Typed-decision question sets are answered by a peer heuristic answerer in wm-core, not by implementing DecisionBackend (pure text classification, no corpus/graph access). Relational questions (store/skip, dedup) need the memory index. Full: @wiki/decisions/heuristic-decision-answering-as-peer-answerer