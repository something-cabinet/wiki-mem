---
title: "EXTRACT-05 Add the three CI gates + crate waiver file"
id: "wiki:tasks:extract-wm-core-05-ci-gates"
type: task
status: done
priority: medium
tags: [from-spec, spec:wm-core-layer-extraction, ci, gates, guard]
acceptance_criteria:
  - text: "AC-3: Gates 1, 2 and 3 are merged and green; Gate 1 asserts the wm-core line ceiling (13,000)"
  - text: "Gate 1: every packages/* member clears 1,500 lines or has a reason in .github/workspace-crates.toml"
  - text: "Gate 2: rmcp, clap, notify, tracing-subscriber, tracing-appender, rust-embed declared only by apps/wm-cli"
  - text: "Gate 3: exactly one pub fn slugify and one frontmatter splitter workspace-wide"
---

schema_version: 1
state: |-
  D-7: three CI gates, because conventions did not hold last time. Gate 1 is a crate floor (1,500 lines) with a checked-in waiver file `.github/workspace-crates.toml`, plus the D-11 wm-core line **ceiling** (13,000, waiver-required above). Gate 2 is a per-layer dependency deny-list. Gate 3 is a singleton-helper grep.

  These gates are the mechanism that makes the boundary enforceable; AC-3 requires them merged and green. Part of @wiki/specs/wm-core-layer-extraction.
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