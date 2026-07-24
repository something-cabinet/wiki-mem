---
id: wiki:specs:rename-knownsid-to-id
title: Rename knowns_id to id in Task Frontmatter
type: spec
tags:
- wiki-maintenance
- naming
- migration
status: approved
relates_to:
- type: creates_task
  target: wiki:tasks:extend-wmpageupdate-to-accept-arbitrary-frontmatter-fields
---
id: wiki:specs:rename-knownsid-to-id

## Overview

All ~164 task wiki files use `knowns_id: <id>` in frontmatter — a legacy artifact from the Knowns import. Rename to `id: <id>` for consistency, and establish that **every wiki page should include an `id` field in frontmatter** matching its canonical page ID.

Currently no code reads `knowns_id` from frontmatter (the `WikiPageMeta.id` is derived from the file path), but the frontmatter field acts as a durable record of the intended identity. Moving it to the canonical `id` key makes it explicit and discoverable.

The graph-connectivity-fix spec (@wiki/specs/graph-connectivity-fix) calls this out in D4: *"Do not index n; it's a legacy Knowns import artifact. A separate migration task should strip it from frontmatter."*

## Locked Decisions

- D1: **Scope — tasks first, then all pages** — Rename `knowns_id` to `id` in all ~164 task files. Separately, add `id` frontmatter to all non-task pages that are missing it.
- D2: **No code changes** — The `Frontmatter` struct has no `knowns_id` field, and `WikiPageMeta.id` comes from file path, not frontmatter. The `id` field will be free-form YAML key like `knowns_id` is today — informational/documentary, not programmatically read.
- D3: **Bulk rename for tasks** — Use sed to replace `knowns_id:` with `id:` across all task files. No per-file review needed — every occurrence is a frontmatter field with identical semantics.
- D4: **Versions unaffected** — `.wm/versions/` history files are immutable and must not be modified.
- D5: **Convention going forward** — `id: <page-id>` becomes standard frontmatter for all new pages created via templates, `wm_page.create`, `wm_task.create`, etc.
- D6: **Bulk script for non-task pages** — Compute the canonical page ID from the file path and insert `id:` via script for all ~290 non-task pages. One-shot batch, no per-page manual work.
- D7: **Lint rule for missing `id`** — Add a check to `wm_lint.check` that flags any page missing `id:` in frontmatter, preventing future omissions.

## Requirements

### Functional Requirements
- FR-1: Every task `.md` file with `knowns_id:` must have it renamed to `id:` in YAML frontmatter
- FR-2: Every non-task wiki page must have `id: <page-id>` added to its frontmatter (if not already present)
- FR-3: Files without `knowns_id` or missing `id` must not be modified beyond adding the `id` field
- FR-4: Non-frontmatter occurrences must not be modified
- FR-5: Page creation tools (`wm_page.create`, `wm_task.create`) and templates must emit `id:` in frontmatter for new pages
- FR-6: `wm_lint.check` must warn on pages missing `id:` frontmatter

### Non-Functional Requirements
- NFR-1: Zero task files left with `knowns_id` in frontmatter
- NFR-2: Zero false positives (no non-frontmatter text modified)
- NFR-3: Wiki health must pass after migration (`wm_lint.check`, `wm_validate.check`)

## Acceptance Criteria

- [ ] AC-1: `rg '^knowns_id:' .wm/wiki/tasks/` returns zero matches
- [ ] AC-2: Every task file has `^id:` in frontmatter
- [ ] AC-3: Every non-task wiki page has `^id:` in frontmatter
- [ ] AC-4: `wm_lint.check` flags pages missing `id:` frontmatter
- [ ] AC-5: `wm_validate.check` passes with no structural issues
- [ ] AC-6: Newly created pages via `wm_page.create` / `wm_task.create` include `id:` in frontmatter

## Scenarios

### Scenario 1: Full task migration
**Given** 164 task files with `knowns_id: <id>` in frontmatter
**When** the rename is applied
**Then** all 164 files have `id: <id>` instead
**And** zero files still contain `knowns_id` in frontmatter

### Scenario 2: Non-task pages get id frontmatter
**Given** a spec, concept, or pattern page without `id:` in its frontmatter
**When** the bulk script runs
**Then** the page gains `id: <canonical-page-id>` in its frontmatter
**And** no other existing frontmatter fields are altered

### Scenario 3: New pages include id
**Given** a new task or page is created via `wm_task.create` or `wm_page.create`
**When** the frontmatter is generated
**Then** it includes `id: <canonical-id>` matching the page's file-path-derived ID

### Scenario 4: Lint catches missing id
**Given** a page without `id:` in frontmatter
**When** `wm_lint.check` is run
**Then** a warning is emitted identifying the page and its file path

## Technical Notes

### Phase 1 — Rename knowns_id to id in tasks

```bash
# Preview: count affected files
rg -l '^knowns_id:' .wm/wiki/tasks/ | wc -l

# Apply replacement (macOS sed)
find .wm/wiki/tasks/ -name '*.md' -exec sed -i '' 's/^knowns_id:/id:/' {} +

# Verify
rg -c '^knowns_id:' .wm/wiki/tasks/
rg -c '^id:' .wm/wiki/tasks/ | wc -l
```

### Phase 2 — Add id frontmatter to all non-task pages

Compute the canonical page ID from each page's file path via `path_to_id()` logic (strip `.wm/wiki/` prefix and `.md` suffix, replace `/` with `:`). Insert `id: <path-derived-id>` at the top of each page's frontmatter (after the opening `---`).

### Phase 3 — Update page creation tools

- `wm_page.create`: add `id: <path>` to generated frontmatter (derived from the `path` param)
- `wm_task.create`: add `id: <task-id>` to generated frontmatter (the task must already compute its ID before writing)
- `wm_template.run`: include `id` in generated frontmatter where applicable

### Phase 4 — Add lint rule

Add a check in `wm_lint.check` (in `apps/wm-core/src/mcp/tools/lint.rs`) that iterates all pages and warns if `id:` is missing from frontmatter — alongside the existing `broken_ref` and `unresolved_target` checks.

### Affected files

| Phase | Pattern | Count | Change |
|-------|---------|-------|--------|
| 1 | `.wm/wiki/tasks/*.md` | ~164 | `knowns_id:` → `id:` |
| 2 | `.wm/wiki/{specs,concepts,patterns,decisions,howto,reference}/*.md` | ~290 | Add `id:` frontmatter |
| 3 | `apps/wm-core/src/mcp/tools/page/action.rs` | 1 | Add `id` to create params |
| 3 | `apps/wm-core/src/mcp/tools/page/mod.rs` | 1 | Emit `id:` in create frontmatter |
| 3 | `apps/wm-core/src/mcp/tools/task/mod.rs` | 1 | Emit `id:` in task create frontmatter |
| 4 | `apps/wm-core/src/mcp/tools/lint.rs` | 1 | Add missing-`id` lint check |

## Open Questions

None — all resolved via D5, D6, D7.