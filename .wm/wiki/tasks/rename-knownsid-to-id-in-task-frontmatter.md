---
title: Rename knowns_id to id in task frontmatter
type: task
tags:
- wiki-maintenance
- naming
- cleanup
status: done
priority: medium
implementation_notes: |-
  Extracted knowledge:
  - @wiki/patterns/bulk-yaml-frontmatter-rename — safe sed-based YAML field renames
  - @wiki/concepts/test-rot-mcp-api-drift — pre-existing test failures from MCP API drift
  - @wiki/decisions/lint-plus-integration-tests-for-wiki-health — two-layer regression guards
  - @wiki/patterns:critical-patterns — 2 entries promoted (test rot + regression guards)
acceptance_criteria:
  - text: "All ~164 task wiki files use id: <id> in frontmatter instead of the legacy knowns_id: <id>"
  - text: "No knowns_id keys remain in task frontmatter and page ID resolution still works (page ID comes from the filename)"
---

schema_version: 1
state: |-
  id: wiki:tasks:rename-knownsid-to-id-in-task-frontmatter

  All ~164 task wiki files use `knowns_id: <id>` in frontmatter (a legacy Knowns import artifact). Rename to `id: <id>` to be consistent with other page types and eliminate the legacy naming.

  The graph-connectivity-fix spec (D4) explicitly calls for stripping `n` from frontmatter. No code changes are needed — the Rust frontmatter parser handles arbitrary YAML keys and the `knowns_id` field is never read by application code (the page ID comes from the filename, not frontmatter).
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
