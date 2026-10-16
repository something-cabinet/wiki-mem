---
title: CWR-03 Three confinement domains + Windows path canonicalisation
type: task
id: "wiki:tasks:cwr-03-confinement-domains-and-windows-canonicalisation"
status: todo
tags: [from-spec, spec:configurable-wm-root, security, windows]
acceptance_criteria:
  - text: Three confinement domains are derived once at engine construction; confine/confine_strict receive the resolved state root for domain 2
  - text: '`../../` and symlink-escape writes against a relocated root are rejected, mirroring the WM-003/WM-004 fixtures'
  - text: 'Windows: canonicalisation, `\\?\` prefix-stripping and case-insensitive comparison happen inside the helpers, with regression tests'
priority: high
---

schema_version: 1
state: |-
  Implements D7 and D16 with AC-7 of @wiki/specs/configurable-wm-root

  - Derive three confinement domains once at engine construction, immutable for the process: (1) project root for source ingestion (`source_dirs`/`source_extensions`), unchanged; (2) the **resolved state root** for wiki/docs/templates/memory/audit log; (3) the global root `$HOME/.wm`.
  - `confine`/`confine_strict` (`apps/wm-core/src/shared/helpers/path_confine_helper.rs:61,87`) receive the resolved root for domain 2. The primitives' logic is untouched: `..` traversal and symlink escapes inside the wiki stay rejected exactly as WM-003/WM-004 enforced them.
  - Windows correctness lives **inside** the helpers, once: canonicalise both root and candidate, strip the extended-length `\\?\` verbatim prefix, and compare case-insensitively (canonicalisation does not normalise case). Doing this at call sites leaves UNC confinement silently over- or under-rejecting.
  - Regression tests in the WM-003/WM-004 fixture style.
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