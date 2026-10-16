---
title: No commit inventories or verification dumps in change descriptions
type: rule
id: "wiki:rules:no-commit-inventories-or-verification-dumps"
status: active
tags: [rule, process, pull-requests, descriptions]
---

schema_version: 1
state: |-
  ## Rule
  Descriptions of a change — pull-request descriptions and commit message bodies — must state **what changed and why**, concisely. They must NOT contain:

  - a commit-by-commit inventory (a "Contents" / "What's included" section that recites commit subjects), or
  - a verification / test-output section (a "Verification" block dumping commands and pass counts).

  ## Why
  A change description is for the reviewer's judgement, not an audit log. Commit subjects are already visible in the changelog, and test evidence is reproducible. Reciting both inflates the description, drifts out of date, and buries the actual summary.

  ## Where the detail belongs instead
  - Verification evidence: the PR/task thread as a comment, or the task implementation notes.
  - Per-commit detail: the commits themselves (`git log`).

  ## Exceptions
  None.
questions:
  - id: enforcement
    type: choice
    instructions: How is this rule enforced?
    options:
    - ci-enforced
    - tool-enforced
    - review-enforced
    - convention-only
  - id: severity
    type: score
    instructions: How severe is a violation of this rule?
    levels:
    - advisory
    - recommended
    - required
    - blocking
  - id: has_exception
    type: noul
    instructions: This rule has documented exceptions.
  - id: applies_to
    type: choice
    instructions: Which surfaces does this rule apply to?
    multi: true
    options:
    - code
    - tests
    - docs
    - configuration
    - workflow
    - security
answers: {}
