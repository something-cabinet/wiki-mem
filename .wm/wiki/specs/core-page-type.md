---
title: Core Page Type — Foundational Project Docs
type: spec
tags:
- spec
- page-type
- core
- type-system
status: approved
---

## Overview

Add `PageType::Core` as a first-class entity in the wiki type system. Core pages are meta-project foundational documents — conventions, architecture, critical patterns, and README — that define *how the project works*. They are automatically loaded at every session init and maintained during extraction.

Core sits alongside the existing 10 types (task, spec, concept, pattern, decision, memory, howto, reference, note, rule) as a plain page type — no additional data struct needed.

## Locked Decisions

- **D1**: Core = meta-project only — reserved for pages about the project itself (conventions, architecture, critical patterns). Nothing domain-specific.
- **D2**: Hybrid init — README always loaded explicitly, then dynamically discover all `type: core` pages via `wm_page.list({"type": "core"})`.
- **D3**: README becomes core (migrated from `reference`).
- **D4**: New Step 7b in wm-extract — after "Promote to Critical", check if extraction qualifies as core. If yes, create as `type: core`.
- **D5**: New `core/` subdirectory under `.wm/wiki/` for core page files.
- **D6**: wm-extract auto-checks — when extracting any knowledge, also scan existing core pages for stale references and suggest updates.

## Requirements

### Functional Requirements

**FR-1: PageType::Core enum variant**
`PageType::Core` exists in the Rust enum with `as_str()` → `"core"`, `allowed_statuses()` → `[Draft, Reviewed, Approved, Archived]`, `priority_rank()` → 9 (highest — core docs are the most important search results).

**FR-2: Core page directory**
`.wm/wiki/core/` is a valid wiki subdirectory. Pages at this path auto-resolve to `PageType::Core` via graph inference.

**FR-3: Page::Core enum variant**
`Core { meta: WikiPageMeta }` variant on the `Page` enum with `From` impls (both directions) — same pattern as Note/Concept/Reference.

**FR-4: wm_page.list filter**
`wm_page.list({"type": "core"})` returns all core pages. `wm_page.list({"type": "core", "status": "active"})` returns only active core pages.

**FR-5: CSS tokens**
`--page-type-core` CSS custom property in light and dark themes in `apps/wm-web/src/styles.css`, alongside the existing 8 page-type tokens.

**FR-6: Page migration**
Four existing pages migrate from their current types to `type: core` and move to `.wm/wiki/core/`:
- `README`: `reference` → `core`
- `ARCHITECTURE`/`WM Architecture`: `reference` → `core`
- `CONVENTIONS`/`WM Conventions`: `reference` → `core`
- `Critical Patterns`: `pattern` → `core`

**FR-7: wm-init core loading**
wm-init Step 3 changes from 3 hardcoded IDs to: read README, then dynamically discover and read all `type: core` pages. The session context summary lists core pages under "Key Docs."

**FR-8: wm-extract Step 7b (promote to core)**
After Step 7 (Promote to Critical), a new Step 7b checks if the extraction qualifies as core:
- Must be meta-project (about the project itself, not domain-specific)
- Must define how work gets done (conventions, architecture, patterns that affect every task)
If yes, create as `type: core` in `core/` subdirectory with a `references` edge back to the source.

**FR-9: wm-extract core staleness check**
When extracting any knowledge, wm-extract scans existing core pages for references that may be stale due to the new extraction. If found, suggests updates to those core pages (does not auto-update — suggests).

### Non-Functional Requirements

- Core pages are markdown with standard YAML frontmatter — no special data struct needed (unlike Rule which has RuleData)
- Core type does not add a new page category to the graph legend — existing rendering treats it as a canonical type with its own CSS token
- Migration of existing pages preserves all content and frontmatter fields except `type`

## Acceptance Criteria

- [ ] AC-1: `PageType::Core` compiles, serializes, deserializes
- [ ] AC-2: `Page::Core { meta }` variant compiles and round-trips through `From` impls
- [ ] AC-3: Core pages can be created, read, updated via existing page CRUD tools
- [ ] AC-4: `wm_page.list({"type": "core"})` returns core pages
- [ ] AC-5: Files at `.wm/wiki/core/*.md` auto-resolve to `PageType::Core`
- [ ] AC-6: `--page-type-core` CSS token renders in both light and dark themes
- [ ] AC-7: README, CONVENTIONS, ARCHITECTURE, critical-patterns migrated to `type: core` in `core/` subdirectory
- [ ] AC-8: wm-init loads all core pages at session start (README + dynamic discovery)
- [ ] AC-9: wm-extract Step 7b exists and can create core pages
- [ ] AC-10: wm-extract scans core pages for staleness during extraction

## Scenarios

### Scenario 1: Session init loads core pages
**Given** the project has 4 core pages (README, CONVENTIONS, ARCHITECTURE, critical-patterns)
**When** wm-init runs
**Then** it loads README, then queries `wm_page.list({"type": "core"})` and reads each core page
**And** the session context summary lists all 4 core pages under "Key Docs"

### Scenario 2: New core page auto-loaded
**Given** a new core page is created (e.g., `core/deployment-conventions`)
**When** the next session initializes
**Then** wm-init automatically discovers and reads it via `wm_page.list({"type": "core"})` — no manual wm-init update needed

### Scenario 3: Extract promotes to core
**Given** a task extracts a new project-wide convention
**When** wm-extract runs and the extracted knowledge is meta-project and foundational
**Then** Step 7b creates it as `type: core` in `core/` with a `references` edge to the source task

### Scenario 4: Extract detects stale core reference
**Given** an extraction changes how the graph model works
**When** wm-extract runs
**Then** it checks ARCHITECTURE (a core page) for stale graph model references and reports any findings

### Scenario 5: Non-core extraction stays in its type
**Given** a task extracts a domain-specific React pattern
**When** wm-extract runs
**Then** it creates as `type: pattern` (not core) — domain-specific patterns are not meta-project

### Scenario 6: Badge renders core color
**Given** a core page is displayed in the page list or search results
**When** the badge renders
**Then** it uses the `--page-type-core` CSS token color, not the gray fallback

## Technical Notes

### Core vs existing types

| Aspect | Core | Reference | Pattern |
|--------|------|-----------|---------|
| Purpose | Meta-project, defines the project | API docs, config tables | Reusable solutions |
| Loaded at init | Yes (dynamic discovery) | No | No |
| Promotion bar | Foundational, meta-project | N/A | Solves a recurring problem |
| Directory | `core/` | `reference/` | `patterns/` |

### Implementation order

1. Rust: `PageType::Core` enum variant
2. Rust: `Page::Core` enum variant + `From` impls
3. CSS: `--page-type-core` tokens
4. Migration: 4 pages → `core/` with `type: core`
5. Graph inference: `core/` dir → `PageType::Core`
6. Skill update: `wm-init` — dynamic core discovery
7. Skill update: `wm-extract` — Step 7b + staleness check
8. Validate: `wm_validate.check()` after migration

### Files to change

| File | Change | Est. Lines |
|------|--------|-----------|
| `packages/wm-engine/src/models/page_type_model.rs` | Add `Core` variant + `as_str`/`allowed_statuses`/`priority_rank` | +7 |
| `packages/wm-engine/src/models/page/page_enum_model.rs` | Add `Core { meta }` variant + match arms in `From` impls | +8 |
| `packages/wm-engine/src/lib.rs` | Add tests for `Core` in test_page_type_priority_rank, test_page_type_as_str | +2 |
| `apps/wm-web/src/styles.css` | Add `--page-type-core` in light + dark sections | +2 |
| `.wm/wiki/core/` (new) | Move 4 pages, update frontmatter `type:` field | +4 |
| `.opencode/skills/wm-init/SKILL.md` | Step 3: dynamic core discovery | +5 |
| `.opencode/skills/wm-extract/SKILL.md` | New Step 7b + staleness check in extract flow | +20 |
| **Total** | | **~48** |

## Open Questions

- (none — all decisions locked)