---
title: 'Failure: Demand inferred from implementation absence'
type: concept
id: "wiki:concepts:failure-demand-inferred-from-implementation-absence"
status: reviewed
tags: [failure, process, measurement, reasoning]
relates_to:
  - {type: references, target: wiki:tasks:curate-01-canonical-global-rule-set}
---

schema_version: 1
state: |-
  ## What went wrong

  An architecture review concluded there was **no cross-project demand** for shared knowledge and recommended deleting the global layer. The evidence it reasoned from was entirely implementation-shaped: zero entries in `$HOME/.wm/wiki/memory/`, zero persisted `layer` values, and one lying write path. The owner's actual complaint — *"we have repeated rules like no comment on so many project"* — was about **rules**, a different page type entirely. The review was correct about the memory artifact and wrong about the requirement.

  ## Root cause

  **Evidence of absence in one artifact was treated as absence of need in the class.** An unbuilt, wrong-typed implementation is not a demand signal — it is evidence about a *design*, not about a *requirement*. The two are easy to conflate when the only data consulted is the feature's own emptiness.

  ## Prevention

  Before accepting a "no demand" verdict, measure the requirement **independently of the implementation** — look for the behaviour the feature would remove. Here that meant asking what the duplicated knowledge actually is: the same rule re-authored across projects. One shell pass then produced the decisive picture — four names for "no comments in code", four for spec-first, and same-name content differing by 4x (498 B vs 2149 B) in different repositories. Prefer measurement across real artifacts over inference from one store's emptiness.

  ## Time lost

  A full spec was written, superseded, and rewritten across three design rounds. The measurement that reversed the verdict took a single command.

  ## Related

  - @wiki/tasks:curate-01-canonical-global-rule-set
  - @wiki/specs/global-wiki-layer
questions:
  - id: kind
    type: choice
    instructions: What kind of concept document is this?
    options:
    - concept
    - failure-analysis
    - research-report
    - reference-note
  - id: category
    type: choice
    instructions: Which domain category does this concept belong to?
    options:
    - architecture
    - search-retrieval
    - graph
    - parser-format
    - mcp-tooling
    - cli
    - storage
    - embeddings
    - web-ui
    - process
  - id: maturity
    type: score
    instructions: How mature is the understanding of this concept?
    levels:
    - raw
    - exploratory
    - established
    - stable
  - id: code_referenced
    type: noul
    instructions: This concept references concrete code.
answers: {}