---
id: wiki:specs:page-type-colored-badges
title: Page-Type Colored Badges
type: spec
status: approved
tags: [spec, ui, badge, colors]
---
id: wiki:specs:page-type-colored-badges


## Overview

The graph view uses 8 `--page-type-*` CSS tokens (concept, spec, task, memory, pattern, decision, howto, reference) with distinct light/dark values. Outside the graph, every page type badge renders as uniform gray (`variant="secondary"`). This spec extends the type color system to all badge surfaces: page list, detail header, search results, and task board.

## Locked Decisions

- D1: **All surfaces** — pages list, detail header, search results, and task board
- D2: **CSS utility classes** in templates (no badge component changes)
- D3: **Gray fallback** for non-canonical types (note, project, etc.)

## Requirements

### Functional Requirements

**FR-1: Page-Type Color Utility**
A helper function or constant that maps a type key to a Tailwind CSS class string using `--page-type-*` tokens:
- `color-mix(in oklch, var(--page-type-{key}) 12%, transparent)` for background
- `var(--page-type-{key})` for text color
- Fallback to `secondary` badge variant for unknown types

**FR-2: Pages List Badges**
The type badge in each page list item (`pages-view.component.ts:168`) must use type-colored class instead of `[variant]="typeBadgeVariant(...)"`.

**FR-3: Detail Header Badge**
The type badge in the page detail header (`pages-view.component.ts:60`) must use type-colored class.

**FR-4: Search Result Badges**
Both type badges in search results (`search-view.component.ts:92-94`) must use type-colored classes.

**FR-5: Task Board Badges**
Type badges in task board cards (`tasks-view.component.ts`) must use type-colored classes.

### Non-Functional Requirements

- NFR-1: Colors must work in both light and dark mode (CSS tokens already handle this)
- NFR-2: No changes to `HlmBadge` component itself
- NFR-3: No performance regression — color-mix is GPU-composited

## Acceptance Criteria

- [ ] AC-1: Pages list shows type badges with distinct colors matching graph legend
- [ ] AC-2: Detail header type badge uses page-type color
- [ ] AC-3: Search result badges show type colors
- [ ] AC-4: Task board card badges show type colors
- [ ] AC-5: Non-canonical types (note, project) render as gray secondary
- [ ] AC-6: Colors update correctly in dark mode (no additional work — CSS tokens already handle this)
- [ ] AC-7: All existing badge functionality preserved (hover, focus, etc.)

## Scenarios

### Scenario 1: User browses pages list
**Given** the pages list shows pages of various types
**When** the user views the list
**Then** each page's type badge shows the correct `--page-type-*` color with tinted background

### Scenario 2: User searches across types
**Given** search results include pages of different types
**When** the user views results
**Then** type badges use matching page-type colors

### Scenario 3: User views a page with non-canonical type
**Given** a page has type "note" (no matching CSS token)
**When** the badge renders
**Then** it falls back to the default `secondary` gray styling

## Technical Notes

### CSS token reference
```css
--page-type-concept: oklch(0.35 0.12 260);   /* dark */
--page-type-concept: oklch(0.65 0.14 260);   /* light */
--page-type-spec: oklch(0.50 0.16 150);
--page-type-task: oklch(0.60 0.18 75);
--page-type-memory: oklch(0.45 0.14 300);
--page-type-pattern: oklch(0.55 0.20 340);
--page-type-decision: oklch(0.45 0.12 185);
--page-type-howto: oklch(0.60 0.18 35);
--page-type-reference: oklch(0.50 0.04 260);
```

### Utility class pattern
```html
<span class="bg-[color-mix(in_oklch,var(--page-type-task)_12%,transparent)] text-[var(--page-type-task)] ...">
  Task
</span>
```

### Implementation locations
- `pages-view.component.ts` — list badge (line ~168), detail badge (line ~60)
- `search-view.component.ts` — result badges (lines ~92-94)
- `tasks-view.component.ts` — task card badges (if type is shown)

### Suggested helper
Extract a function `pageTypeBadgeClass(type: string): string` that returns the utility class or falls back to empty string (for `hlmBadge variant="secondary"` to apply defaults).

## Open Questions

- [ ] Should `pageTypeBadgeClass` live in `graph-color.service.ts` (alongside PAGE_TYPES) or in a new utility?
- [ ] Do tasks on the task board display a type badge at all? Check current template.
