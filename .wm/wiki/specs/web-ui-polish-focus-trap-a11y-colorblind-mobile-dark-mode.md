---
title: Web UI Polish — Focus Trap, A11y, Colorblind, Mobile, Dark Mode
type: spec
tags:
  - spec
  - approved
  - web-ui
  - accessibility
---
id: wiki:specs:web-ui-polish-focus-trap-a11y-colorblind-mobile-dark-mode

## Overview

Fix 9 Web UI polish items identified during designer review. These span accessibility, responsive design, visual indicators, and interaction quality for the SvelteKit web frontend.

## Locked Decisions

- D1: Focus trap implemented as Svelte action (not component) for reusability
- D2: Colorblind indicators use symbols alongside color (not replacing color)
- D3: Mobile table overflow uses scrollable container + responsive card fallback at 640px
- D4: Graph tooltip HTML must be escaped before rendering
- D5: Reduced motion disables skeleton shimmer, not full animations

## Requirements

### Functional Requirements

- FR-1: Dialog focus must be trapped within the dialog (Tab/Shift+Tab cycle)
- FR-2: Task cards must be keyboard accessible (Space + Enter)
- FR-3: Blocked tasks must participate in status cycling
- FR-4: Priority and status must have symbol/text indicators, not just color
- FR-5: Tables must be horizontally scrollable on mobile
- FR-6: Graph node tooltips must not render raw HTML
- FR-7: Focus rings must be clearly visible (30% opacity minimum)
- FR-8: Skeleton shimmer must respect prefers-reduced-motion
- FR-9: Toasts must announce to screen readers via aria-live

### Non-Functional Requirements

- NFR-1: No regressions in existing page interactions
- NFR-2: All Svelte build steps complete without errors

## Acceptance Criteria

- [ ] AC-1: Focus trap action exists in `src/lib/actions/focusTrap.ts` and is applied to ConfirmDialog, HelpOverlay, and all modals
- [ ] AC-2: Task cards (tasks/+page.svelte) respond to Space key with same behavior as Enter
- [ ] AC-3: Task cards have `aria-label` for screen readers
- [ ] AC-4: Status cycle includes `blocked: 'todo'` and statusDisplay includes `blocked: 'blocked'`
- [ ] AC-5: Priority indicators show symbols (⚠●○) and status columns show text labels
- [ ] AC-6: Sources table (sources/+page.svelte) is in a scrollable container; below 640px shows card layout
- [ ] AC-7: GraphView.svelte escapes HTML in node titles before rendering in vis-network tooltips
- [ ] AC-8: `app.css` focus ring box-shadow opacity is at least 30%
- [ ] AC-9: `@media (prefers-reduced-motion: reduce)` disables skeleton shimmer in app.css
- [ ] AC-10: Toast.svelte has `aria-live="polite"` on the toast container

## Scenarios

### Scenario 1: Keyboard user navigates dialogs
**Given** a dialog is open (ConfirmDialog, HelpOverlay)
**When** user presses Tab
**Then** focus cycles through dialog elements only; pressing Tab on the last element returns to the first

### Scenario 2: Mobile viewport
**Given** a narrow viewport (<640px)
**When** viewing the sources table
**Then** horizontal scrolling is available, or each row renders as a card

### Scenario 3: Reduced motion preference
**Given** user has `prefers-reduced-motion: reduce` enabled
**When** the page loads and content is being fetched
**Then** skeleton placeholders appear without shimmer animation

## Technical Notes

- All file paths relative to wm-ui/src/
- Previous dark mode work (task 94qxox) set up CSS variables — this spec extends that foundation
- focusTrap action should use `data-focus-trap` attribute for focusable element detection
- For colorblind symbols: ⚠=high, ●=medium, ○=low for priority; use text labels for status columns
