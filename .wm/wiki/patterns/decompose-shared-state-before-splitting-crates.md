---
title: "Decompose shared state before splitting coupled modules into crates"
type: pattern
id: "wiki:patterns:decompose-shared-state-before-splitting-crates"
status: reviewed
tags: [pattern, architecture, domain-driven, refactor, crates]
relates_to:
  - {type: references, target: wiki:specs:engine-state-decomposition}
---

schema_version: 1
state: |-
  ## Problem

  You want to split a set of modules into separate crates, but they all read and write one shared state object (a god-object). Every attempted split fails the closed-interface test: each new crate's public API still exposes or requires the whole shared state, so the split adds ceremony without decoupling.

  ## Solution

  Decompose the shared state first, then split:

  1. Turn the god-object into a struct of per-domain handles - one owner per domain, explicit cross-domain methods, no sibling field access. Keep it a facade so call sites do not all change at once, and migrate one domain per change.
  2. Only then decide, per domain, whether a crate is earned - against an explicit rubric (mass, dependency-extraction value, multi-consumer).

  The signal that this is the right order: a single shared-state type is reachable from every module in a cycle. Until each module has a closed interface, no split is real. Splitting the *application* layer (the callers) is the wrong axis too - the domains are underneath it, not in it.

  ## When to Use

  A large crate where several modules reference each other through one state/context type, and a crate split is being proposed.

  ## When Not to Use

  When the modules already have closed interfaces (split directly), or when the state is genuinely one cohesive aggregate - then it is a domain object, not a god-object.

  ## Related

  - @wiki/specs/engine-state-decomposition
  - @wiki/specs/wm-core-layer-extraction
questions:
  - id: problem_kind
    type: choice
    instructions: What kind of problem does this pattern solve?
    options:
    - architecture
    - api-design
    - data-model
    - error-handling
    - performance
    - testing
    - ui
    - tooling
    - workflow
  - id: preconditions_required
    type: noul
    instructions: This pattern requires specific preconditions to be met.
  - id: complexity
    type: score
    instructions: How complex is applying this pattern?
    levels:
    - trivial
    - simple
    - moderate
    - complex
  - id: language_specific
    type: noul
    instructions: This pattern is specific to a programming language.
answers: {}