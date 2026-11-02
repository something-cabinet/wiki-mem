---
title: 'Next session: finish the v0.6.1 release and settle the open decisions'
type: task
id: "wiki:tasks:next-session-finish-the-v061-release-and-settle-the-open-decisions"
status: todo
priority: urgent
tags: [handoff, release]
acceptance_criteria:
  - text: The owner has chosen how to handle zip RUSTSEC-2026-0429 and cargo audit exits 0
  - text: The local commits are pushed, v0.6.1 points at a green commit, and the CI check, publish and publish-npm jobs pass
  - text: npm view @something-cabinet/wm-cli version returns 0.6.1
  - text: The global --dry-run spec has its open questions answered and is approved
  - text: The uncommitted eightcap-new-portal files are committed, discarded or kept on purpose
  - text: The Kiro wm MCP connection is restored and wm_initial works
  - text: Implementation of wm-cli migrate has started, tests first
---

schema_version: 1
state: |-
  Handoff for the next session. State on 2026-11-02: origin/master and the v0.6.1 tag both point at 2099c3d, which is red (the CI check job fails there). Four local commits fix the gates and the zstd advisories (e576be1, a26a8cc) and record the specs; they and the commit of this extraction are not pushed.

  Open decisions for the owner:
  1. zip RUSTSEC-2026-0429 (pinned by candle-core): A ignore it in .cargo/audit.toml like the three earlier zip advisories (recommended), B wait for candle-core, C drop the decision feature. Spec: wiki:specs:clear-the-new-cargo-audit-advisories.
  2. Global --dry-run flag, spec wiki:specs:wm-cli-global-dry-run-flag-for-write-commands: option A, B or C for the implementation; whether wm_page and wm_task get a dry_run argument; keep or remove health audit --dry-run and migrate-records --apply as aliases; exit code of a dry run that finds problems.
  3. The eightcap-new-portal working tree holds uncommitted copies of WIKI-MEM.md and .kiro/steering/wiki-mem.md, the deletion of .kiro/steering/knowns.md, and untracked execution-rules.md and workflow-rules.md. Commit, discard or keep them there.

  Approved and waiting for implementation: wm-cli migrate (wiki:specs:wm-cli-migrate-upgrades-the-whole-wiki-with-versioned-records), tests first. It repairs the 12 stub-frontmatter pages in eightcap-new-portal and fixes the memory-import data loss.

  Facts to know:
  - The Kiro wm MCP connection is down; reconnect it. Until then drive the tools over stdio (wiki:howto:drive-the-wm-mcp-tools-over-stdio-from-a-script).
  - .kiro/knowledge disappeared from eightcap-new-portal on 2026-11-02 and the cause is unknown.
  - Pre-existing clippy failures, not touched: wm-embed vector_db.rs chunks_exact, two lints in wm-core tests, an unused call_err helper in wm-tools tests.
  - Before pushing, run wiki:howto:run-the-ci-check-job-locally-before-pushing.
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
