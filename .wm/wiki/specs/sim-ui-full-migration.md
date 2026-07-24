---
id: wiki:specs:sim-ui-full-migration
title: Sim UI Full Migration Spec
type: spec
status: draft
tags: [spec, angular, ui, sim-ui, migration]
relates_to:
  - {type: references, target: wiki:specs:tauri-desktop-migration}
  - {type: references, target: wiki:tasks:srv-wire-angular-to-http--replace-tauri-ipc-with-fetch}
---
id: wiki:specs:sim-ui-full-migration

## Overview

Migrate every UI pattern in the Angular web app to [Sim UI](https://simui.dev) components — an Angular component library built on Spartan UI + Tailwind CSS. Sim UI provides 500+ component variants across 23 categories. Where Sim UI has no equivalent (sidebar, sheet, tooltip, spinner, graph canvas), keep the current spartan-helm or custom implementation.

This supersedes the old `sim-ui-component-integration` spec (now marked superseded).

## Design Principle: Elements Must Fit Their Role

Several current patterns use components that are **semantically incorrect** for their role. Every visual element should communicate its purpose through its component choice, not just its styling.

| Current Pattern | Used As | But Should Be | Because |
|---|---|---|---|
| `wmBtn` toggle group | Search filter tabs | Sim UI **Tabs** | Buttons trigger actions; tabs switch between content views/filter states |
| Raw `<p class="text-destructive">` | Error messages | Sim UI **Alert** | Plain text has no icon, no role="alert", no dismiss — not accessible as feedback |
| Raw `<button>` with card styling | Result/task/page items | Sim UI **Card** | A clickable card is a card, not a button. Cards provide structure (header/content/footer) |
| Raw `<button>` with sun/moon icon | Dark mode toggle | Sim UI **Switch** | A toggle is a binary state control with on/off semantics, not an action button |
| No component at all | Create/edit/delete feedback | Sim UI **Notification** | Users need persistent, dismissible feedback confirming their actions |
| Raw `<kbd>`, raw empty states | Visual patterns | Sim UI styled | Each view has its own inconsistent empty/loading/error pattern |

## Locked Decisions

- D1: Sim UI copy-paste model — components are copied into `src/libs/ui/`, not npm-installed
- D2: Spartan UI handles headless/accessible primitives, Sim UI provides the visual layer
- D3: Tailwind CSS stays as the styling engine
- D4: Components without Sim UI equivalents keep their current implementation (sidebar, sheet, tooltip, spinner, graph)
- D5: `@ng-icons/lucide` stays — Sim UI uses the same icon set

## Requirements

### Functional Requirements

- FR-1: Replace `wmBtn` with Sim UI Button (54 variants) across all 6 views + layout
- FR-2: Replace `wmInput` with Sim UI Input (55 variants) in Search, Memory, Pages
- FR-3: Replace `wmDialog` with Sim UI Dialog (35 variants) in Memory, Pages
- FR-4: Replace `wmSelect` with Sim UI Select (39 variants) in Memory, Pages
- FR-5: Replace `wmBadge` with Sim UI Badge (23 variants) across all 6 views
- FR-6: Replace `wmCard` with Sim UI Card (11 variants) in Memory, Settings, Graph, and raw card patterns
- FR-7: Replace `wmAccordion` with Sim UI Accordion (22 variants) in Tasks view
- FR-8: Add Sim UI Notification (33 variants) as a toast system for create/edit/delete feedback
- FR-9: Replace raw error/success alert divs with Sim UI Alert (25 variants)
- FR-10: Replace search type filter toggle with Sim UI Tabs (20 variants)
- FR-11: Replace dark mode toggle buttons with Sim UI Switch (18 variants)
- FR-12: Delete or deprecate old `wm-` component source files after migration

### Non-Functional Requirements

- NFR-1: Existing layout and responsive behavior must be preserved
- NFR-2: No regressions in existing functionality (search, navigation, CRUD)
- NFR-3: Component source is owned in-repo (copy-paste model)
- NFR-4: Accessibility must be maintained or improved (Spartan UI is WAI-ARIA compliant)
- NFR-5: Build must pass with zero errors after migration
- NFR-6: All existing E2E journeys must pass

## Sim UI Component Availability

| Component | Sim UI Variants | Status |
|---|---|---|
| **Button** | 54 | ✅ Replace `wmBtn` + raw buttons |
| **Input** | 55 | ✅ Replace `wmInput` |
| **Dialog** | 35 | ✅ Replace `wmDialog` |
| **Select** | 39 | ✅ Replace `wmSelect` |
| **Badge** | 23 | ✅ Replace `wmBadge` |
| **Card** | 11 | ✅ Replace `wmCard` + raw card patterns |
| **Accordion** | 22 | ✅ Replace `wmAccordion` |
| **Alert** | 25 | ✅ Replace raw error divs |
| **Notification** | 33 | ✅ NEW — toast system |
| **Tabs** | 20 | ✅ Replace search filter toggle |
| **Switch** | 18 | ✅ Replace dark mode toggle |
| **Breadcrumb** | 9 | ✅ Optional: layout top bar |
| **Sidebar** | — | ❌ Keep `hlmSidebar` |
| **Sheet** | — | ❌ Keep `hlmSheet` |
| **Tooltip** | — | ❌ Keep `hlmTooltip` |
| **Spinner** | — | ❌ Keep `wmSpinner` |
| **Graph canvas** | — | ❌ Keep custom WebGL graph |

## Acceptance Criteria

- [ ] AC-1: Sim UI Button replaces all `wmBtn` usage (search, memory, pages, settings, type filter)
- [ ] AC-2: Sim UI Input replaces all `wmInput` usage (search, memory, pages forms)
- [ ] AC-3: Sim UI Dialog replaces all `wmDialog` usage (create/edit/delete modals)
- [ ] AC-4: Sim UI Select replaces all `wmSelect` usage (memory filters, page type picker)
- [ ] AC-5: Sim UI Badge replaces all `wmBadge` usage (type badges, counts, status)
- [ ] AC-6: Sim UI Card replaces `wmCard` and raw card divs (result cards, task cards, page items, settings panels)
- [ ] AC-7: Sim UI Accordion replaces `wmAccordion` (task board columns)
- [ ] AC-8: Sim UI Alert replaces all raw error/success divs (5+ instances across all views)
- [ ] AC-9: Sim UI Notification added — toast on create/edit/delete success and error
- [ ] AC-10: Search type filter uses Sim UI Tabs instead of wmBtn toggle
- [ ] AC-11: Dark mode toggle in Settings + layout footer uses Sim UI Switch
- [ ] AC-12: Old `wm-` component files are deleted or deprecated
- [ ] AC-13: Build passes (`ng build`)
- [ ] AC-14: All E2E journeys pass
- [ ] AC-15: Designer reviews and approves component choices before merge

## Per-View Migration Map

### Search (`search-view.component.ts`)

| Element | Current | Target | Role Fix? |
|---|---|---|---|
| Search input | `wmInput` | Sim UI Input | — |
| Search button | `wmBtn` default | Sim UI Button | — |
| Type filter | `wmBtn` toggle | Sim UI Tabs | ✅ Pills → proper tab semantics |
| Result cards | Raw `<a>` Tailwind | Sim UI Card | ✅ Div → proper card component |
| Result count | Raw `<span>` badge | Sim UI Badge | — |
| Error state | Raw `<div>` | Sim UI Alert | ✅ Plain div → accessible alert |
| Loading | `wmSpinner` + text | Keep (no Sim UI spinner) | — |
| Empty state | Raw `<div>` | Sim UI styled | ✅ |

### Tasks (`tasks-view.component.ts`)

| Element | Current | Target | Role Fix? |
|---|---|---|---|
| Column headers | `wmAccordion` | Sim UI Accordion | — |
| Task cards | Raw `<button>` | Sim UI Card | ✅ Button → proper card |
| Error state | Raw `<div>` | Sim UI Alert | ✅ |
| Badges | `wmBadge` | Sim UI Badge | — |
| Loading | `wmSpinner` | Keep | — |
| Empty state | Raw `<div>` | Sim UI styled | ✅ |

### Memory (`memory-view.component.ts`)

| Element | Current | Target | Role Fix? |
|---|---|---|---|
| Dialogs (3) | `wmDialog` | Sim UI Dialog | — |
| Filters (2) | `wmSelect` | Sim UI Select | — |
| Form inputs | `wmInput` | Sim UI Input | — |
| Entry cards | `wmCard` | Sim UI Card | — |
| Tags | `wmBadge` | Sim UI Badge | — |
| Action buttons | `wmBtn` | Sim UI Button | — |
| "Show more/less" | Raw `<button>` | Sim UI Button link | ✅ |
| Error text | Raw `<p>` | Sim UI Alert | ✅ |
| Loading | `wmSpinner` | Keep | — |

### Pages (`pages-view.component.ts`)

| Element | Current | Target | Role Fix? |
|---|---|---|---|
| Dialogs (3) | `wmDialog` | Sim UI Dialog | — |
| Type select | `wmSelect` | Sim UI Select | — |
| Form inputs | `wmInput` | Sim UI Input | — |
| Page list items | Raw `<button>` | Sim UI Card | ✅ Button → proper card |
| Badges | `wmBadge` | Sim UI Badge | — |
| Action buttons | `wmBtn` | Sim UI Button | — |
| Error messages | Raw `<p>` | Sim UI Alert | ✅ |
| Content display | Raw `<pre>` | Sim UI Card styled | — |
| Loading | `wmSpinner` | Keep | — |

### Graph (`graph-view.component.ts`)

| Element | Current | Target | Role Fix? |
|---|---|---|---|
| Stats badges | `wmBadge` | Sim UI Badge | — |
| Hover tooltip | `wmCard` | Sim UI Card | — |
| Error state | Raw `<div>` | Sim UI Alert | ✅ |
| Empty state | Raw `<div>` | Sim UI styled | ✅ |
| Loading | `wmSpinner` | Keep | — |

### Settings (`settings-view.component.ts`)

| Element | Current | Target | Role Fix? |
|---|---|---|---|
| Status cards | `wmCard` | Sim UI Card | — |
| Badges | `wmBadge` | Sim UI Badge | — |
| Buttons | `wmBtn` | Sim UI Button | — |
| Error state | Raw `<div>` | Sim UI Alert | ✅ |
| Dark mode toggle | `wmBtn` outline | Sim UI Switch | ✅ Button → proper toggle |
| Loading | `wmSpinner` | Keep | — |

### Layout (`layout.component.ts`)

| Element | Current | Target | Role Fix? |
|---|---|---|---|
| Sidebar | `hlmSidebar` | Keep (no Sim UI alternative) | — |
| Footer dark mode | Raw `<button>` | Sim UI Switch | ✅ Button → proper toggle |
| Top bar | Raw `<header>` | Keep or Sim UI Breadcrumb | Optional |

## Implementation Order

1. **Sim UI Button** — most used, touches every view. Copy Button variants needed (default, outline, ghost, destructive, link, icon)
2. **Sim UI Badge + Card** — replaces `wmBadge`, `wmCard`, and all raw card patterns
3. **Sim UI Input + Select + Accordion** — replaces form inputs and task board
4. **Sim UI Dialog** — replaces all modals in Memory + Pages
5. **Sim UI Tabs** — replaces search filter toggle
6. **Sim UI Switch** — replaces dark mode toggles
7. **Sim UI Alert** — replaces all raw error divs (quick win, touches all views)
8. **Sim UI Notification** — adds toast system (new capability)
9. **Cleanup** — delete/deprecate old `wm-` source files
10. **Designer review** — approve component choices before final merge

## Open Questions

- [ ] Which Sim UI Button variants to use for each use case (search, edit, delete, create, icon-only)?
- [ ] Toast position (top-right vs bottom-right vs top-center)?
- [ ] Which Alert variants for error vs success vs warning?
- [ ] Should Sim UI Breadcrumb replace the raw top-bar header in layout?
