---
id: wiki:concepts:web-ui-ux-principles
title: "Web UI UX Principles"
type: concept
tags: [ux, ui, web, frontend, design]
status: draft
relates_to:
  - {type: references, target: wiki:decisions:replace-hardcoded-colors-with-css-variables}
  - {type: references, target: wiki:tasks:web-ui-ux-audit-fix}
  - {type: references, target: wiki:patterns:systematic-ux-audit-methodology}
---
id: wiki:concepts:web-ui-ux-principles

# Web UI UX Principles

## Overview

Systematic UX principles used for evaluating and fixing the WM Web UI. Based on established UX laws and HCI research.

## Principles Checklist

### 1. Consistency
Similar elements must look and behave the same across all views.

**WM Web:** Errors use consistent `bg-destructive/10 border-destructive/20 rounded-lg` across all views. All buttons use `wmBtn`.

### 2. Hierarchy
Clear visual hierarchy guides the user's eye.

**WM Web:** Page titles use `text-xl sm:text-2xl font-bold`. Section headers use `text-sm uppercase tracking-wider text-muted-foreground`.

### 3. Proximity (Gestalt)
Related items visually grouped with adequate gaps.

**WM Web:** Empty/error states use cards. Task columns have clear spacing. Search uses `space-y-2`.

### 4. Affordance
Clickable items must look clickable.

**WM Web:** All clickables use `wmBtn` or `cursor-pointer`. Back button fixed to `wmBtn variant="ghost"`.

### 5. Fitts's Law
Primary actions should be large enough and easy to reach.

**WM Web:** Primary action buttons use `variant="default"`. Retry buttons added on error states.

### 6. Hick's Law
Choices presented clearly without overwhelming.

**WM Web:** Search initial state shows guidance text. Type filters use toggle buttons.

### 7. Jakob's Law
Follow familiar UI patterns.

**WM Web:** Standard sidebar nav, search bar at top, cards for content, dialogs for forms.

### 8. Accessibility
Keyboard nav, screen readers, reduced motion.

**WM Web:** `:focus-visible` rings. `@media (prefers-reduced-motion: reduce)` query. `aria-label` attributes. `(keydown.enter)` handlers.

### 9. Color/Contrast
Sufficient contrast, semantic color usage.

**WM Web:** All hardcoded colors replaced with CSS variable theme tokens. Adapts to light/dark modes automatically.

### 10. Spacing/Rhythm
Consistent padding, margins, and whitespace.

**WM Web:** Consistent `p-6 max-w-4xl mx-auto` layout. Dialog forms use `space-y-3`. Cards use `p-4`/`p-5`.

## Related
- @wiki/decisions/replace-hardcoded-colors-with-css-variables
- @wiki/tasks/d5cc21
- @wiki/patterns/systematic-ux-audit-methodology
