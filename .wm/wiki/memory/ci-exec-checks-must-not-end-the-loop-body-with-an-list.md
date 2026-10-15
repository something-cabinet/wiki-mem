---
title: CI -exec checks must not end the loop body with an && list
type: memory
tags: [ci, shell, testing, failure]
status: active
---

schema_version: 1
state: |-
  A POSIX `find ... -exec sh -c '<loop>' sh {} +` CI check whose loop body ends in an `&&` list (for example `[ "$n" -gt "$max" ] && echo ...`) returns a non-zero status for every in-budget item. `find -exec ... {} +` propagates a non-zero status from any invocation, so the command substitution fails and, under GitHub Actions' default `bash -e`, the step aborts — the check is permanently red even with zero violations. A red-always check is worse than no check because it trains people to ignore the job.
  Fix: make the loop body's final command status-neutral, e.g. `if [ "$n" -gt "$max" ]; then echo "..."; fi`, and keep the pass/fail decision in the outer `[ -z "$violations" ] || { ...; exit 1; }`.
  Verified in the wm-cli file-size budget check (`.github/workflows/ci.yml`, the `check` job): the `&&` form exits 1 with no output; the `if/then` form exits 0 when clean and exits 1 with a file list when the threshold is lowered. Source: @wiki/tasks:controlled-rewrite-of-appswm-clisrcmainrs--one-architecture-decision-per-review.
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
