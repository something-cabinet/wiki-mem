---
{}
relates_to:
  - {type: references, target: wiki:tasks:add-pagetypecore-enum-variant-and-pagecore-enum-variant}
---

---
title: Pattern: Page Type Registration — Complete Touch Points
type: pattern
id: wiki:patterns:page-type-registration-touch-points
tags: [pattern, type-system, enum, parser]
---

## Problem

Adding a new `PageType` variant to the enum is not enough — there are multiple scattered code locations where the type must be registered. Missing any one causes silent misclassification (pages rendered as `concept` instead of the intended type).

## Solution

When adding a new `PageType`, update ALL of the following touch points:

### 1. Core enum definition
`packages/wm-engine/src/models/page_type_model.rs` — Add variant, `as_str()`, `priority_rank()`

### 2. Page enum variant
`packages/wm-engine/src/models/page/page_enum_model.rs` — Add variant with `From` impls for both directions

### 3. String-based parser mapping
`apps/wm-core/src/parser/mod.rs` — Add to `parse_page_type()` match. **This is the most commonly missed step.**

### 4. MCP tool filter (list)
`apps/wm-core/src/mcp/tools/page/mod.rs` — Add to both the `WmPageAction::List` filter and the `WmPageAction::Create` path-inference match

### 5. Lint auto-fixer
`apps/wm-core/src/graph/lint.rs` — Add to `inferred` match in `auto_fix_missing_frontmatter()`

### 6. Reference resolver
`apps/wm-core/src/reference_service.rs` — Add to the `ref_type` match that resolves `@wiki/{type}/{name}` references

### 7. CSS token (frontend)
`apps/wm-web/src/styles.css` — Add `--page-type-{name}` in both light and dark themes

### 8. Test setup directories
`apps/wm-core/tests/helpers/setup.rs` and `apps/wm-core/tests/file_watcher_test.rs` — Add to the list of wiki `create_dir_all()` calls

### Verification
After adding the type, verify with:
- `wm_page.list({"type": "<new-type>"})` returns the expected pages
- Graph stats (`wm_graph_stats`) shows the new type with the expected count

## When to Use
When adding any new `PageType` variant to the system.

## When Not to Use
For changes to page statuses, edge types, or other non-PageType enumerations — each has its own registration pattern.

## Related
- @wiki/patterns:rule-as-page-type — Prior example of adding a PageType with structured data
- @wiki/specs:core-page-type — Spec that motivated this pattern (the parse_page_type miss was discovered during implementation)