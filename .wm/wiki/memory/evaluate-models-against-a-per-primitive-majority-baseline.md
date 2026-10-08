---
title: Evaluate models against a per-primitive majority baseline
type: memory
tags: [eval, baseline, uncertainty, model-selection]
status: active
---

schema_version: 1
state: |-
  Eval methodology: judge a candidate model against a PER-PRIMITIVE majority baseline (aggregates hide losses), use Wilson CIs (treat within-noise spreads as no effect), feed the model's trained channels (label_descriptions/examples), separate latency levers from accuracy levers, test the purpose-built model as an endpoint, persist per-config JSON. Full: @doc/patterns/eval-model-against-per-primitive-majority-baseline
questions:
  - id: layer
    type: choice
    instructions: Which memory layer does this entry belong to?
    options:
    - project
    - global
    - session
  - id: store_or_skip
    type: noul
    instructions: This entry is worth storing as durable memory.
  - id: dedup_action
    type: choice
    instructions: How should this entry relate to existing memory?
    options:
    - new
    - merge
    - supersede
    - skip
  - id: confidence
    type: score
    instructions: How confident is the recorded knowledge?
    levels:
    - low
    - medium
    - high
answers: {}
