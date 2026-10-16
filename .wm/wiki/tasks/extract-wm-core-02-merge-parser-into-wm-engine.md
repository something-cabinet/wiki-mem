---
title: "EXTRACT-02 Merge parser/ into wm-engine as wm_engine::parser"
id: "wiki:tasks:extract-wm-core-02-merge-parser-into-wm-engine"
type: task
status: done
priority: high
tags: [from-spec, spec:wm-core-layer-extraction, refactor, wm-engine, parser]
acceptance_criteria:
  - text: "parser/ (1,558 lines) is a non-breaking submodule wm_engine::parser; wm-code-intel compiles unchanged and its tests pass"
  - text: "Frontmatter is collapsed into WikiPageMeta; the duplicate page-type-to-string mapping is deleted"
  - text: "cargo build, test and clippy --workspace are green"
---

schema_version: 1
state: |-
  D-6 step 1: merge `parser/` into `wm-engine` as `wm_engine::parser`. Uniting parser with the models makes the second copy unrepresentable rather than merely deleted. Collapse `Frontmatter` (~74 fields) into `WikiPageMeta` and delete one page-type mapping.

  Before step 2 so the tools' `crate::parser::` imports already point at the new home. Keep it a non-breaking submodule addition so `wm-code-intel` (which depends on wm-engine) does not move; rerun code-intel's tests if `WikiPageMeta` fields change. Part of @wiki/specs/wm-core-layer-extraction.
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