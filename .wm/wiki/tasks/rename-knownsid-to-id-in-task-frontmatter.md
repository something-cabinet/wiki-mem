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
---

id: wiki:tasks:rename-knownsid-to-id-in-task-frontmatter

All ~164 task wiki files use `knowns_id: <id>` in frontmatter (a legacy Knowns import artifact). Rename to `id: <id>` to be consistent with other page types and eliminate the legacy naming.

The graph-connectivity-fix spec (D4) explicitly calls for stripping `n` from frontmatter. No code changes are needed — the Rust frontmatter parser handles arbitrary YAML keys and the `knowns_id` field is never read by application code (the page ID comes from the filename, not frontmatter).