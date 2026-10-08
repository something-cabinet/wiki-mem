---
title: NC-01 Ban all comments + strip rustdoc
type: task
id: "wiki:tasks:nc-01-ban-all-comments--strip-rustdoc"
status: done
priority: high
tags: [from-spec, spec:ban-all-comments, policy, p1]
spec: specs/ban-all-comments
acceptance_criteria:
  - text: "Rule no-comments-in-code bans rustdoc; doc-comment-convention void"
  - text: "CONVENTIONS updated (no rustdoc exemption)"
  - text: "No comment lines remain in production .rs code"
  - text: "cargo check/clippy/tests clean"
---

schema_version: 1
state: |-
  Ban all comments including rustdoc: update rules + CONVENTIONS, then strip existing doc comments workspace-wide. Per specs/ban-all-comments. Depends on RW-01.
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
