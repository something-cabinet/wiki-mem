---
title: Don't conflate Knowns' features — code-gen vs platform config are separate
type: memory
tags: [failure, research, knowns]
status: active
---

schema_version: 1
state: |-
  When researching Knowns patterns, don't conflate their code generation system (knowns template run, uses Handlebars) with their platform config generation (built with Go map literals). These are different features with different needs. Verify which feature you're comparing. Reference: @wiki/concepts/handlebars-hbs-rabbit-hole
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
