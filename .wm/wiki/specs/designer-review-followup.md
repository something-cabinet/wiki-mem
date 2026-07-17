---
title: Designer Review Follow-up — UI Polish
type: spec
tags: [spec, angular, ui, responsive, polish]
---

## Overview

Resolve the 8 findings from the designer's strict review of the Sim UI Angular components. These cover responsive gaps, theme fragmentation, design system consistency, and CSS issues.

## Style Keywords (Design Vocabulary)

**Minimal · Neutral · Accessible · Softly rounded · Subtly tactile**

All styling decisions should align with these keywords. No bold brand colors, no gradients, no sharp corners. Prioritize accessibility and subtle tactile feedback (press scale, hover shadows, smooth transitions).

## Locked Decisions

- D1: Sidebar navigation icons must be replaced with `<ng-icon>` or direct SVG components (not `[innerHTML]`)
- D2: Layout must use CSS variable theming (`bg-sidebar`, `text-sidebar-foreground`) instead of hardcoded slate/gray classes
- D3: `wm-dialog` must be used for the Pages create modal instead of the custom overlay
- D4: `wm-card` must have default padding (`p-5`)

## Requirements

### Functional Requirements

- FR-1: Sidebar renders SVG icons correctly (not stripped by Angular sanitizer)
- FR-2: Sidebar participates in the CSS variable theming system
- FR-3: Graph view stat cards use responsive grid: `grid-cols-1 sm:grid-cols-2`
- FR-4: Memory view filter toolbar wraps on mobile (`flex-wrap`)
- FR-5: Pages view create modal uses the design system's `wm-dialog` component
- FR-6: Clickable type filters in Search view use `wmBtn` pill variant instead of `wmBadge`
- FR-7: View headings scale with viewport (`text-xl sm:text-2xl`)
- FR-8: `wm-card` includes default padding (`p-5`)
- FR-9: Investigate and resolve 32 CSS selector errors from `@spartan-ng/brain/hlm-tailwind-preset.css`

### Non-Functional Requirements

- NFR-1: Build must pass with zero errors
- NFR-2: All existing E2E journeys must pass
- NFR-3: Layout must work on mobile (320px+) and desktop

## Acceptance Criteria

- [ ] AC-1: Sidebar icons render in the browser (SVGs visible)
- [ ] AC-2: Sidebar uses `bg-sidebar`, `text-sidebar-foreground` CSS variable classes
- [ ] AC-3: Graph stats use `grid-cols-1 sm:grid-cols-2`
- [ ] AC-4: Memory toolbar wraps on narrow viewports
- [ ] AC-5: Pages create modal uses `<wm-dialog>` component
- [ ] AC-6: Search type filters use buttons with proper hover/focus/active states
- [ ] AC-7: Headings use responsive text sizing
- [ ] AC-8: `wm-card` renders with default padding
- [ ] AC-9: Build passes with zero errors
- [ ] AC-10: All 14 E2E journeys pass

## Scenarios

### Scenario 1: Sidebar on Mobile
**Given** a user on a mobile device (viewport <768px)
**When** the page loads
**Then** the sidebar is hidden (off-screen)
**And** a hamburger button is visible
**And** tapping the hamburger slides in the sidebar with icons visible and correct colors

### Scenario 2: Create Page Modal
**Given** a user on the Pages list view
**When** they click "Create Page"
**Then** a `wm-dialog` opens with backdrop blur and smooth scale/fade animation
**And** the dialog uses the design system's theme variables

### Scenario 3: Graph Stats Responsive
**Given** a user on the Graph view
**When** on a mobile viewport
**Then** stat cards stack vertically (single column)
**When** on a desktop viewport
**Then** stat cards display in 2 columns

## Implementation Order

1. **wm-card padding** (quickest, affects many views)
2. **Sidebar SVGs + theming** (most impactful visual fix)
3. **Graph responsive grid** (simple grid class change)
4. **Memory toolbar flex-wrap** (single class addition)
5. **Pages dialog** (swap custom modal for wm-dialog)
6. **Search filter badges → buttons** (change component + style)
7. **Responsive headings** (add sm: breakpoint prefixes)
8. **CSS selector errors investigation** (research task)

## Technical Notes

### Sidebar SVG Fix
The layout component at `src/app/layout/layout.component.ts` uses `[innerHTML]="item.icon"` which Angular sanitizes. Replace with direct SVG markup or use `<ng-icon>` from `@ng-icons/core` (already used by Sim UI).

### Layout Theming
Current classes like `bg-slate-900`, `text-slate-400`, `bg-gray-50` should become `bg-sidebar`, `text-sidebar-foreground`, `bg-background`.

### Pages Modal
Current code has a manual `fixed inset-0 bg-black/40` overlay. Replace with `<wm-dialog [isOpen]="showCreateForm" (close)="showCreateForm = false">` wrapping the form content.

### CSS Selector Errors
32 rules skipped with `& -> Empty sub-selector` — likely a Tailwind v4 compatibility issue with the spartan preset. Check if updating `@spartan-ng/brain` or the preset resolves it.

### Card Padding
Add `p-5` to the default classes in `wm-card.ts`. Consumers can override via the `class` input.

## Open Questions

- [ ] Should the investigation of 32 CSS selector errors be a separate task?
- [ ] Should responsive typography use `text-xl md:text-2xl` or a different scale?
