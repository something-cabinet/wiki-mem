---
title: 'Decision: All standard wiki pages are structured records'
type: decision
id: "wiki:decisions:all-wiki-pages-structured-records"
status: draft
tags: [decision, structured, typed-decisions, wiki, migration]
relates_to:
  - {type: implements, target: wiki:specs:structured-all-doc-types}
---

schema_version: 1
state: |-
  ## Context

  After the typed-decision record format landed for descriptive docs, we asked whether **all** wiki pages should be structured. The decision model that motivated the format proved **non-viable**, but the format's value is independent: uniform, machine-checkable docs.

  ## Decision

  Make **every standard page type** a record (`{schema_version, state, questions, answers}`) and **rewire prose consumers to read `state`**. Exclude `index.md` (generated) and non-standard dirs (`learnings`/`conventions`/`research`); allow a `record_format: prose` opt-out for pathological bodies.

  ## Rationale

  Uniform validation/processing, and it structurally eliminated the duplicate-frontmatter and escaped-newline bug classes. Consumers (`wm_page.get`, memory read, task description, `wm_search.query`) read `state`, so agents still receive prose; the validator and migration planner share **one directory-based record scope** so excluded pages aren't errored.

  ## Consequences

  744+ pages converted (160 descriptive + 584 others). 16 pages remain prose **by design**: `index.md`, 11 `learnings`, `conventions`, `research`, 2 opted-out specs. `wm_validate` all-scope = 0 errors.
questions:
  - id: outcome
    type: choice
    instructions: What is the recorded outcome of this decision?
    options:
    - adopted
    - rejected
    - deferred
    - superseded
    - abandoned
  - id: reversibility
    type: noul
    instructions: The decision can be reversed cheaply without data migration or cross-module breakage.
  - id: confidence
    type: score
    instructions: How strong is the recorded justification for the selected outcome?
    levels:
    - low
    - medium
    - high
  - id: impact
    type: choice
    instructions: How wide is the blast radius of this decision?
    options:
    - local
    - component
    - system
    - project-wide
answers: {}