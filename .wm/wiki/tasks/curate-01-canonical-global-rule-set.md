---
title: '[CURATE-01a] Curation map + scope decisions (executable now)'
type: task
id: "wiki:tasks:curate-01-canonical-global-rule-set"
status: done
tags: [curation, rules, global]
acceptance_criteria:
  - checked: true
    text: A curation map records canonical slug and merged text for every global candidate, resolving each same-name-different-content pair to a single text
  - checked: true
    text: All 46 rule pages across the four projects carry an explicit scope decision (global, project-local, or retire); draft and superseded pages are excluded from global
  - checked: true
    text: AGENTS.md is ported by deliberate mapping; wm performs no path read of a per-user file
  - checked: true
    text: The ordering and staging decision (empty global root first) is recorded
priority: high
time_started: 2026-10-15T10:10:44.913409+00:00
relates_to:
  - {type: implements, target: wiki:specs:global-wiki-layer}
time_spent: 0h 5m
---

schema_version: 1
state: |-
  Phase A of CURATE-01 - steps 1-4 of the plan approved 2026-10-15. Phase B is wiki:tasks:curate-01b-populate-global-root, blocked on the composition mechanism.

  1. **Curation map.** Canonical slug plus merged text for each global candidate, absorbing the measured clusters (no-comments 4 names, spec-first 4, tdd 3, no-else 2, no-magic-values 2, parallel-execution 2) and the singleton global candidates. Resolve each same-name-different-content pair to a single text. Deliverable: one reference page. (AC-1, AC-2)

  2. **Scope decision for all 46 pages** across the four projects: global, project-local, or retire. Known project-local residue: eightcap's five e2e-* rules (~16 KB), prefer-nested-scss, respect-file-tree-and-barrel-files, app-code-conventions; axon's ui-changes-design-system-first; wiki-mem's rust-anti-patterns. no-allow-attributes (draft) and doc-comment-convention (superseded) must not go global. (AC-1, AC-2)

  3. **Port /Users/nhkhanh/projects/AGENTS.md** (5,907 B): map each section to an existing canonical rule, a new canonical rule, or explicitly out of scope. A deliberate owner port; wm performs no path read of a per-user file. (AC-3)

  4. **Record the ordering and staging decision** - mechanism ships with an empty global root, curation populates it afterwards; staging stays inside this repository because confinement blocks out-of-root writes until the mechanism exists. (AC-4)

  Everything here is executable now and writes nothing outside this repository.
questions:
  - id: work_kind
    type: choice
    instructions: What kind of work is this task?
    options:
    - feature
    - bugfix
    - refactor
    - docs
    - test
    - chore
    - migration
  - id: priority
    type: choice
    instructions: What priority is this task?
    options:
    - low
    - medium
    - high
    - urgent
  - id: needs_spec
    type: noul
    instructions: This task depends on a spec.
  - id: has_ac
    type: noul
    instructions: This task has at least one acceptance criterion.
answers: {}