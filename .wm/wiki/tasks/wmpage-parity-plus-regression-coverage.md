---
title: wm_page parity plus regression coverage
type: task
id: wiki:tasks:wmpage-parity-plus-regression-coverage
status: done
priority: medium
tags:
- from-spec
- spec:wm-doc-type-frontmatter
- wm-doc-fix-02
spec: wiki:specs:wm-doc-type-frontmatter
acceptance_criteria:
- text: 'AC-4: Parity test — wm_doc.create and wm_page.create with identical inputs produce byte-identical frontmatter for type'
  checked: true
- text: 'AC-5: Existing wm_page, wm_doc, and MCP suite tests pass unchanged'
---

schema_version: 1
state: |-
  Add parity + regression tests: identical inputs through wm_doc.create and wm_page.create must produce byte-identical frontmatter (type included); all existing wm_page/wm_doc/MCP suites must pass unchanged. Follow the inproc harness pattern (tests/helpers/inproc.rs) like mcp_test.rs:217/249. From spec wiki:specs:wm-doc-type-frontmatter (FR-4, AC-4/5).
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
