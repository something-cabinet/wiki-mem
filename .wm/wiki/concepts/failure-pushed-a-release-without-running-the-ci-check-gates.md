---
title: 'Failure: pushed a release without running the CI check gates'
type: concept
id: "wiki:concepts:failure-pushed-a-release-without-running-the-ci-check-gates"
status: reviewed
tags: [failure, ci, release, gates]
relates_to:
  - {type: relates_to, target: wiki:tasks:wm-core-exceeds-its-13000-line-gate-1-ceiling-after-the-code-index-work}
---

schema_version: 1
state: |-
  ## What went wrong

  I pushed 13 commits and force-moved the v0.6.1 tag. The CI check job failed, so publish and publish-npm never started. Tests, clippy and rustfmt had all passed locally, but they are not the check job.

  ## Root cause

  The check job also enforces, in this order:

  - a 300-line budget per wm-cli source file (80 for main.rs);
  - a 13000-line ceiling on apps/wm-core/src (Gate 1);
  - a dependency deny-list per layer that counts dev-dependencies (Gate 2);
  - one definition per singleton helper (Gate 3);
  - the library unit suites without the onnx feature;
  - cargo audit.

  My changes broke three of them: health_service.rs grew to 322 lines, wm-core reached 13030 lines because unit tests sit inside src, and a log-capture test added tracing-subscriber to wm-engine. New RUSTSEC advisories on the zstd and zip crates then failed the audit step. The wm-core ceiling had already failed once before (wiki:tasks:wm-core-exceeds-its-13000-line-gate-1-ceiling-after-the-code-index-work).

  A parity test that read .kiro/steering/wiki-mem.md would have failed too: .kiro/ is gitignored here, so a fresh clone does not have the file.

  ## Prevention

  - Run the whole check job locally before any push or tag. The commands are in wiki:howto:run-the-ci-check-job-locally-before-pushing.
  - Put new tests for wm-core in apps/wm-core/tests, not in src, because only src counts toward the ceiling.
  - Do not add a dev-dependency to a package that the deny-list does not allow. Move the test to a crate that may use it.
  - Never test against a gitignored path.
  - Re-run cargo audit on the day of a release. The advisory database changes without any commit of yours.

  ## Time lost

  About an hour and a half, a tag force-moved onto a red commit, and a release that is still blocked.
questions:
  - id: kind
    type: choice
    instructions: What kind of concept document is this?
    options:
    - concept
    - failure-analysis
    - research-report
    - reference-note
  - id: category
    type: choice
    instructions: Which domain category does this concept belong to?
    options:
    - architecture
    - search-retrieval
    - graph
    - parser-format
    - mcp-tooling
    - cli
    - storage
    - embeddings
    - web-ui
    - process
  - id: maturity
    type: score
    instructions: How mature is the understanding of this concept?
    levels:
    - raw
    - exploratory
    - established
    - stable
  - id: code_referenced
    type: noul
    instructions: This concept references concrete code.
answers:
  kind: failure-analysis
  category: process
  maturity: established
  code_referenced: true