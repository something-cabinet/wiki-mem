---
title: wm-doc — Typed Pages + Edges Foundation
type: spec
status: draft
tags: [spec, wm-doc, typed-pages, edges, foundation]
---

## Overview

Currently 12 of 15 wm skills create pages, but only wm-spec uses typed pages (`wm_page.create` with `type` param) and edges (`wm_page.link`). The other 11 all use flat `wm_doc.create`. Root cause: **wm-doc** — the canonical doc-creation reference skill — only teaches flat-doc creation.

Update wm-doc to teach typed page creation with edges, making it the foundation for all other skills to adopt the pattern.

## Locked Decisions

- D1: Keep `wm_doc.create` for non-typed pages (backward compat). Add typed-page section alongside.
- D2: Reference `@wiki/concepts/edge-types` for canonical edge list — don't inline all 16.
- D3: Only foundation (wm-doc) specced here. Individual skill upgrades are separate follow-ups.

## Requirements

### FR-1: Add "Creating Typed Pages" section
Document `wm_page.create` with `type` parameter and per-type data:

| PageType | Directory | `type` param | Per-type data |
|---|---|---|---|
| Task | `tasks/` | `"task"` | task_data.acceptance_criteria, estimate |
| Spec | `specs/` | `"spec"` | spec_data.functional_requirements, goals |
| Decision | `decisions/` | `"decision"` | decision_data.context, options, rationale, outcome |
| Pattern | `patterns/` | `"pattern"` | pattern_data.when_to_use, example |
| Concept | `concepts/` | `"concept"` | — (meta only) |
| Memory | `memory/` | `"memory"` | memory_data.layer, ttl_days |
| Howto | `howto/` | `"howto"` | — (meta only) |
| Reference | `reference/` | `"reference"` | — (meta only) |
| Note | `notes/` | `"note"` | — (meta only) |

### FR-2: Add "Linking Pages with Edges" section
Document `wm_page.link` for typed relationships:
```json
wm_page.link({"id": "wiki:specs/my-spec", "target": "wiki:tasks/my-task", "edge_type": "implements"})
```
Include SDD-relevant edge table + link to `@wiki/concepts/edge-types`.

### FR-3: Update "Reading Pages" section
Add `wm_page.get` alongside `wm_doc.get` — typed pages return structured frontmatter data.

### FR-4: Update existing Doc Types table
Add note: "For typed pages (decision, task, pattern, spec), use `wm_page` instead — supports `type` param and per-type data."

### FR-5: Keep backward compat
Existing `wm_doc.create` content unchanged. Typed-page section is additive.

## Acceptance Criteria

- [ ] AC-1: wm-doc SKILL.md has "Creating Typed Pages" section with `wm_page.create` docs and page-type table.
- [ ] AC-2: wm-doc SKILL.md has "Linking Pages with Edges" section with `wm_page.link` docs.
- [ ] AC-3: wm-doc SKILL.md references `@wiki/concepts/edge-types`.
- [ ] AC-4: wm-doc SKILL.md mentions `wm_page.get` for typed page reading.
- [ ] AC-5: Existing `wm_doc.create` content unchanged.
- [ ] AC-6: Build passes (SKILL.md is embedded).

## Scenarios

### Scenario 1: wm-extract upgrades
After wm-doc is updated, wm-extract can create typed Decision/Pattern/Concept pages instead of flat docs, referencing wm-doc as its guide.

### Scenario 2: New skill starts typed
A developer writing a new wm-* skill reads wm-doc, finds the "Creating Typed Pages" section, and uses typed pages from the start.

## Technical Notes

- File: `apps/wm-core/src/skills/wm-doc/SKILL.md` and `.claude/skills/wm-doc/SKILL.md`
- Page-type table must match canonical `PageType` enum
- No code changes — pure SKILL.md update
