---
title: 'Pattern: Supersede with the reason of record'
type: pattern
id: "wiki:patterns:supersede-with-the-reason-of-record"
status: reviewed
tags: [pattern, process, supersession, documentation]
relates_to:
  - {type: references, target: wiki:decisions:no-global-rule-layer-per-project-rules}
---

schema_version: 1
state: |-
  ## Problem

  A decision gets reversed. The status flips to `superseded` and **the reasoning evaporates**. The next session sees a dead page, cannot tell why it died, and re-proposes the same thing - which is how a codebase oscillates on the same architecture twice.

  This repository did exactly that: a 12-package extraction was drafted, a collapse of over-split crates was then executed, and the extraction was revived four months of context later. Nothing recorded *why* the first attempt failed, so the third attempt had to rediscover it.

  ## Solution

  A supersession records four things, not one:

  1. **Status** - `superseded` on the artifact.
  2. **The reason of record** - why it was reversed, in the owner's own words where the call was theirs. Not "no longer needed".
  3. **What survives**, and why it still holds. Often most of the artifact is still correct.
  4. **What is withdrawn**, listed explicitly - the specific claims, contracts or steps that no longer apply.

  And critically: **annotate the superseded artifact itself.** A page marked `approved` or `done` that has been reversed must say so where a reader will land on it, because status alone does not carry the reversal into someone else's reading. A completed task stays `done` when it was correct for what it did - annotate it rather than reopening it.

  Applied five times on 2026-10-15/16: `global-wiki-layer` (reason: the global rule set is the owner's decision), `global-rules-cascade`, the curation map, `extract-packages-from-wm-core` (never executed; the overpromise that got it collapsed), and the `e688f0` collapse task (annotated `Reversed`, left `done`).

  ## When to Use

  Every reversal of an approved spec, an accepted decision, or a completed task whose conclusion no longer holds. Also when reviving something previously killed - the revival must state which of the old objections it answers.

  ## When Not to Use

  Nothing substantive was decided or built - a draft nobody acted on. Still mark the status, but a sentence is enough.

  ## Related

  - @wiki/decisions:no-global-rule-layer-per-project-rules
  - @wiki/specs:wm-core-layer-extraction
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