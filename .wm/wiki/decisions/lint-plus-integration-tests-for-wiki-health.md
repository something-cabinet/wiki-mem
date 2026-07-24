---
{}
relates_to:
  - {type: references, target: wiki:tasks:rename-knownsid-to-id-in-task-frontmatter}
---

---
title: Decision: Lint + Integration Tests as Regression Guards for Wiki Health
type: decision
status: approved
tags: [decision, testing, lint, regression, wiki-health]
---

## Context

After renaming `knowns_id` → `id` in task frontmatter across 444 wiki pages, we needed a way to prevent regression. The migration was a one-time bulk change, but future pages created via `wm_page.create` or `wm_task.create` must also include `id:` in frontmatter. Without a guard, the property would silently degrade over time.

## Decision

Add two layers of regression protection:

1. **Lint check** (`wm_lint.check`): Iterates all graph nodes, reads each page's raw frontmatter, and warns if `^id:` is absent. Runs on demand via CLI or MCP.
2. **Integration tests**: Four new tests in `mcp_test.rs` that verify:
   - `wm_lint.check` catches missing `id:` on a page without it
   - `wm_lint.check` passes for pages with `id:`
   - `wm_page.create` emits `id:` in generated frontmatter
   - `wm_task.create` emits `id:` in generated frontmatter

This follows the existing pattern: `wm_lint.check` already warns on orphans, unresolved targets, missing ACs, and draft specs.

## Rationale

- **Lint alone is not enough** — it's only run on demand. A test that runs in the test suite catches the regression during development, not after deployment.
- **Tests alone are not enough** — they test specific scenarios. The lint check is the safety net for all pages, including existing ones that might be edited manually.
- **Two layers** with different trigger conditions: lint catches existing issues, tests catch new regressions during development.

## Consequences

- `wm_lint.check` output size increases slightly (one issue per page missing `id:`)
- ~30 lines of test code maintainence burden per test
- Pre-existing wiki pages that already have `id:` are not affected
- The lint check reads raw file content for every graph node — acceptable at current scale (524 nodes, file reads are fast)

## Related

- @wiki/tasks/rename-knownsid-to-id-in-task-frontmatter
- @wiki/specs/rename-knownsid-to-id