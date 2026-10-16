---
title: 'Pattern: Preference artifacts present options, not decisions'
type: pattern
id: "wiki:patterns:preference-artifacts-present-options-not-decisions"
status: reviewed
tags: [pattern, process, authority]
relates_to:
  - {type: answers, target: wiki:concepts:failure-agent-decided-owner-preferences}
---

schema_version: 1
state: |-
  ## Problem

  An artifact exists so the owner can decide - which conventions to share, which variant is canonical, what belongs to everyone versus this project. An agent fills it in with resolutions instead of options and quietly takes over the owner's authority.

  ## Solution

  Separate **the evidence** from **the ruling** in the artifact itself:

  1. Present the measured facts: what exists, what diverges, what each variant contains.
  2. List candidate options per item with the trade-off each carries, and mark one as a recommendation - explicitly labelled as one.
  3. Leave the decision field blank. Never fill it in.
  4. State what changes under each option, so the owner can decide without another round.

  The test: if the artifact would still be **correct** after the owner flips every recommendation, it is an options document. If flipping one makes it inconsistent, the agent made a decision.

  ## When to Use

  Any deliverable whose content is the owner's taste or authority rather than a derived fact: shared conventions, naming and style choices, which rules bind everyone, what is public versus private, brand and copy, org policy.

  ## When Not to Use

  Genuinely convergent problems, which have a verifiable right answer. Preferring a stack-agnostic variant over a project-specific one is a fact about generality; deciding whether that variant *should* be shared is a preference. Optimising a query, fixing a path bug, correcting a record body: just do it.

  ## Related

  - @wiki/concepts:failure-agent-decided-owner-preferences
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