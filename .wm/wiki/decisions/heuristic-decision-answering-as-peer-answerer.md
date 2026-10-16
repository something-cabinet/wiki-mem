---
title: 'Decision: Heuristic typed-decision answering is a peer answerer in wm-core'
type: decision
id: "wiki:decisions:heuristic-decision-answering-as-peer-answerer"
status: approved
tags: [decision, typed-decisions, heuristics, architecture]
relates_to:
  - {type: implements, target: wiki:tasks:implement-memory-write-decision-gate--heuristic-answering-of-the-memory-question-set}
---

## Context
The canonical typed-decision question sets needed a deterministic answerer after the local model runtime (gliner-rs) was declared non-viable (0.491 vs a 0.623 majority baseline).

## Decision
Answer question sets with a peer heuristic answerer in `wm-core` (in-process with graph, search, and config), producing `DecisionResult`-shaped answers directly. Leave `DecisionBackend`/`DecisionRuntime` in `wm-engine` untouched.

## Rationale
`DecisionBackend::classify(text, specs)` is pure text classification with no corpus or graph access, but memory questions (store/skip, dedup) are relational and need the memory index. Routing them through the classification trait would be a compensating layer and would fabricate calibrated probabilities. A peer answerer is a second honest producer of the same output type.

## Consequences
Heuristic answering shares the question model and answer codec, not the classification pipeline. The model path remains available behind the `decision` feature for a future fine-tuned model.

## Related
@wiki/tasks/implement-memory-write-decision-gate--heuristic-answering-of-the-memory-question-set
@wiki/specs/memory-write-decision-gate