---
title: "Web UI: UX audit and fix pass"
type: task
status: done
tags: [ui, web, angular, ux, review]
priority: medium
---

# Web UI: UX audit and fix pass

## Description

Thorough UX audit of the entire WM Web UI (`apps/wm-web/`) against established UX laws (Fitts's, Hick's, Gestalt, Jakob's) and UI principles (consistency, hierarchy, affordance, contrast). Fixed all issues found across 9 files.

## Changes

### Global (styles.css, index.html)
- Added `:focus-visible` ring for keyboard navigation accessibility
- Added `@media (prefers-reduced-motion: reduce)` support for accessibility
- Added `::-webkit-scrollbar` theming for aesthetic consistency
- Added `system-ui` to font stack for broader compatibility
- Added `theme-color` meta tags for mobile browser chrome
- Updated title and description meta tags
- Added `antialiased` class to body

### Layout (layout.component.ts)
- Compacted top header `h-14` to `h-11` for better proportion
- Improved sidebar footer version label readability

### Search view
- Added initial empty state with guidance text and keyboard hint
- Fixed hardcoded error colors (bg-red-50 -> bg-destructive/10)
- Added `(keydown.enter)="doSearch()"` for keyboard support

### Graph view
- Wrapped error state in themed card with shadow
- Improved empty state with helpful message
- Added card container for better visual hierarchy

### Tasks view
- Fixed hardcoded error display (text-destructive -> card)
- Added colored dot indicators to status headers for Gestalt grouping
- Refined header background opacity values

### Pages view
- Fixed Back button affordance (raw button -> wmBtn ghost)
- Replaced inline spinner CSS with wm-spinner component
- Fixed hardcoded badge colors (bg-violet-50, bg-amber-50 -> theme vars)

### Memory view
- Added WmSpinner import, replaced inline spinner
- Fixed hardcoded error colors (text-red-500 -> text-destructive)
- Fixed all 7 tag colors (bg-violet-50, bg-amber-50, bg-cyan-50, bg-orange-50 -> theme vars)

### Settings view
- Added error display with themed card and Retry button
- Added aria-label to dark mode toggle
- Fixed layout stability with min-width on toggle button

## UX Principles Applied

| Principle | Application |
|-----------|------------|
| Consistency | All errors use `bg-destructive/10 border-destructive/20 rounded-lg` |
| Affordance | Back button uses wmBtn; all clickables have cursor-pointer |
| Hierarchy | Proper opacity levels on status headers |
| Proximity (Gestalt) | Cards group related info in empty/error states |
| Fitts's Law | Retry buttons and primary actions sized appropriately |
| Hick's Law | Search initial state reduces cognitive load with guidance |
| Accessibility | Focus-visible rings, reduced-motion, aria-labels, keyboard Enter |

## Files Changed (9 files, 806 insertions, 280 deletions)
- apps/wm-web/src/styles.css
- apps/wm-web/src/index.html
- apps/wm-web/src/app/layout/layout.component.ts
- apps/wm-web/src/app/views/search/search-view.component.ts
- apps/wm-web/src/app/views/graph/graph-view.component.ts
- apps/wm-web/src/app/views/tasks/tasks-view.component.ts
- apps/wm-web/src/app/views/pages/pages-view.component.ts
- apps/wm-web/src/app/views/memory/memory-view.component.ts
- apps/wm-web/src/app/views/settings/settings-view.component.ts

## Build Verification
- `ng build` passes with no errors
- Only pre-existing warning about `regl` ESM module
