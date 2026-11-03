---
title: Clear the new cargo audit advisories so CI can publish
type: task
id: "wiki:tasks:clear-the-new-cargo-audit-advisories-so-ci-can-publish"
status: in-progress
priority: urgent
tags: [security, dependencies, ci]
spec: wiki:specs:clear-the-new-cargo-audit-advisories
acceptance_criteria:
  - text: The zstd, zstd-safe and zstd-sys advisories are fixed by a lockfile bump
    checked: true
  - text: cargo audit exits 0 on master
    checked: true
  - text: The CI check job is green on master and on the v0.6.1 tag, so publish can start
implementation_notes: |-
  Owner chose option A for zip RUSTSEC-2026-0429 on 2026-11-02: ignore it in .cargo/audit.toml like the three earlier zip advisories and extend the removal task. Remaining: apply the ignore, confirm cargo audit exits 0, then push and read the CI run.
  Added RUSTSEC-2026-0429 to .cargo/audit.toml. cargo audit exits 0; gates 1-3 and wm-cli file-size budget pass locally. Lib test suite was still compiling (cold build). Uncommitted; push and CI result pending.
implementation_plan: |-
  1. Add RUSTSEC-2026-0429 to the ignore list in .cargo/audit.toml (spec option A, owner decision 2026-11-02). No new comments.
  2. Run cargo audit; it must exit 0.
  3. Extend wiki:tasks:remove-zip-audit-ignores-when-candle-core-allows-zip--9 to cover 0429.
  4. Run the CI check job gates locally (per critical pattern 2026-11-02).
  5. On owner request only: commit, push, and watch the check job on master and the v0.6.1 tag.
---

schema_version: 1
state: |-
  The v0.6.1 tag and master runs fail in the check job. Gates fixed in e576be1. cargo audit still reports zstd, zstd-safe, zstd-sys (fixed by a lockfile bump, done) and zip RUSTSEC-2026-0429 (pinned by candle-core, waiting for the owner's choice between options A, B and C in the spec).
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