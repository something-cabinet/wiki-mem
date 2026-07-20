---
title: Sim UI Designer Review
type: spec
status: reviewed
tags: [spec, designer, review, sim-ui, web-ui]
---

## Overview

All custom `wm-` components and raw-Tailwind UI patterns have been replaced with [Sim UI](https://simui.dev) components (built on Spartan UI + Tailwind CSS v4). The designer needs to review visual quality, component choice, and role correctness across all 6 views + layout.

### Current Status

- **Review completed:** 2026-07-20 — 44 pass / 2 fail / 2 partial
- **Findings (P1–P3):** Captured from tauri-pilot run — status confirmed by designer review (see results page)
- **Designer Decisions:** 8 questions resolved + verified against code
- **Results tracked at:** `sim-ui-designer-review-results`
- **Tasks needed:** P1 Accordion, P1 Select overlay, P2 column colors, N1–N3 (see results page)

### Checklist Process

The designer reviews each checklist item and marks **pass** or **fail**. Fails become new tasks. Items marked **N/A** are not applicable.

**Review performed 2026-07-20 by AI designer sub-agent (ng serve with mock-injector.js).** For re-review instructions, see "How to Review" below.

## Locked Decisions

- D1: Review is **partial** — P1–P3 findings captured + 8 designer decisions resolved, but the per-view checklist still needs a designer to complete.
- D2: This spec is the **visual verification checklist**. The companion `designer-review-followup` spec covers **prior design review fixes** (sidebar SVGs, wm-card padding, theme class migration) — it is a separate, earlier review pass, not the implementation plan for this spec's P1–P3 findings. P1–P3 fixes get their own tasks (see results page).
- D3: Each checklist item gets **pass/fail**. Fails create new tasks.
- D4: P1–P3 findings have **per-item status** (table below).
- D5: P1 Accordion expanded default, P1 Select dropdown overlay, and P3 Graph empty state are **unfixed**. P2 Column distinction **needs designer input**.
- D6: Task creation for unfixed findings waits for the designer review — documented findings in this spec are sufficient until then.
- D7: The 8 "Resolved" designer decisions were quick-verified against the codebase before the checklist review.
- D8: Pass/fail results are tracked on a **separate page** (`sim-ui-designer-review-results`), not inline in this spec.

## What Changed

### Replaced Components

| Old | New Sim UI | Variants Available |
|---|---|---|
| `wmBtn` (custom button) | `hlmBtn` (Sim UI button) | 54 |
| `wmInput` | `hlmInput` | 55 |
| `wmDialog` | `hlmDialog` (`BrnDialog` + `hlm-dialog-*`) | 35 |
| `wmSelect` | `hlmSelect` (`BrnSelect` + `hlm-select-*`) | 39 |
| `wmBadge` | `hlmBadge` | 23 |
| `wmCard` | `hlmCard` | 11 |
| `wmAccordion` | `hlmAccordion` (`hlm-accordion-*`) | 22 |
| Raw error divs | `hlmAlert` | 25 |
| Raw dark mode button | `hlmSwitch` | 18 |
| Search type filter (wmBtn toggle) | Sim UI Tabs | 20 |
| No feedback system | `ngx-sonner` toast (via `ToastService`) | — |

### Kept As-Is (No Sim UI Equivalent)

| Component | Reason |
|---|---|
| `hlmSidebar` + sub-components | No Sim UI sidebar |
| `hlmSheet` | No Sim UI sheet/drawer |
| `hlmTooltip` | No Sim UI tooltip |
| `wmSpinner` | No Sim UI spinner |
| `CanvasGraphDirective` | Custom WebGL graph — not UI |
| `@ng-icons/lucide` | Sim UI uses the same icons |

## Screenshot Findings (From tauri-pilot run)

Screenshots captured at `tauri-pilot-screenshots/*-final.png`.

### P1 — Tasks: All Accordion Columns Expanded
All task board status columns (draft, todo, in-progress, etc.) are expanded by default, making the page extremely tall (1MB screenshot). Should start collapsed and expand on trigger click.

**Fix:** Set `[isOpened]="false"` on `hlm-accordion-item` by default. The accordion trigger handles toggle automatically via BrnAccordionTrigger.

**Status:** 🔴 Not yet fixed

### P1 — Memory: Select Dropdown Extends Header Instead of Overlaying
The `<hlm-select-content>` renders inline, pushing the header button row down and breaking the layout. It should render as a floating overlay.

**Fix:** Wrap `<hlm-select-content>` inside `<hlm-select-portal>` so the dropdown renders overlaying content below, not pushing it.

**Status:** 🔴 Not yet fixed

### P2 — Tasks: Column Visual Distinction
All status columns look visually similar — the colored header bars and dot indicators help distinguish status categories. Worth checking if the color mapping is distinct enough for quick scanning.

**Status:** 🟡 Needs designer input (evaluate during checklist review)

### P3 — Graph: Empty/Loading States
Graph view shows a canvas with no nodes — the empty state message should be centered and visible. The canvas background color needs checking.

**Status:** 🔴 Not yet fixed

## Designer Decisions (Resolved)

All 8 questions answered by designer review. Changes applied.

| # | Question | Decision | Status | Verified |
|---|---|---|---|---|
| 1 | Search type filter -> Tabs? | **Yes, Sim UI Tabs** | Wired in | ✅ `HlmTabs` with 4 options, `(tabActivated)` wired to `doSearch()` |
| 2 | Task cards -> hlmCard? | **Yes, hlmCard** | Swapped | ✅ `hlmCard` on each task item in tasks view |
| 3 | Memory tag colors -> Badge variants? | **Yes, use Badge variants** | Removed hash color logic | ✅ `hlmBadge variant="secondary"` used; no hash-based color logic found |
| 4 | Button variants per role? | **Standard mapping**: Search/Submit = default, Cancel = ghost, Edit = outline, Delete = destructive, Icon-only = ghost | Already matches current usage | ✅ All 5 variants (`default`, `destructive`, `ghost`, `outline`, `link`) actively used |
| 5 | Toast position? | **Top-right** (keep current) | No change needed | ✅ `NgxSonnerToaster position="top-right"` in layout component |
| 6 | Alert variants? | **destructive=error, default=success, outline=warning** | Matches current usage | ⚠️ `destructive` used for errors; `default` is implicit (no variant attr = default); `outline` variant not available in HlmAlert — non-error alerts use `default` |
| 7 | Wire Tabs in Search? | **Yes, wire now** | Done | ✅ `hlmTabs [tab]` bound to `searchType`, `(tabActivated)` triggers search |
| 8 | Dark mode v3 class audit? | **Yes, scan and fix** | No v3 gray classes found | ✅ Zero matches for `bg-gray-`, `text-gray-`, `bg-slate-`, `text-slate-` across all templates |

## Review Checklist

### Per-View

#### Search (`/search`)
- [ ] Search input (`hlmInput`) — correct styling, placeholder visible
- [ ] Search button (`hlmBtn`) — proper default variant
- [ ] Type filter — Sim UI Tabs working correctly for switching search type
- [ ] Result cards (`hlmCard`) — correct card appearance with hover state
- [ ] Result count badge (`hlmBadge`) — positioned correctly
- [ ] Error state — Alert component visible when error occurs
- [ ] Empty state — layout and spacing acceptable
- [ ] Loading state — spinner + text

#### Tasks (`/tasks`)
- [ ] Accordion columns start collapsed (not expanded)
- [ ] Accordion expand/collapse animation smooth
- [ ] Task cards (`hlmCard`) — correct card appearance with priority border
- [ ] Status column colors distinguishable (see P2)
- [ ] Badges (`hlmBadge`) on column headers and priorities — variant choice correct
- [ ] Error state — Alert visible

#### Memory (`/memory`)
- [ ] Select dropdowns overlay (not push content down)
- [ ] Create/Edit/Delete dialogs (`BrnDialog` + `hlm-dialog-*`) — animation, backdrop, close button
- [ ] Layer/Status filter selects — trigger appearance, dropdown, option selection
- [ ] Form inputs (`hlmInput`) — proper styling
- [ ] Entry cards (`hlmCard`) — correct appearance with expand/collapse
- [ ] Tag badges (`hlmBadge`) — variant="secondary" consistently
- [ ] Action buttons (`hlmBtn`) — ghost/edit/delete variants correct
- [ ] "Show more/less" — `hlmBtn` link variant
- [ ] Error states — Alert visible

#### Pages (`/pages`)
- [ ] Create/Edit/Delete dialogs — animation, backdrop, close button
- [ ] Type select — Sim UI select overlays correctly
- [ ] Page list cards (`hlmCard`) — clickable card appearance
- [ ] Badges (`hlmBadge`) — type badge variants
- [ ] Content view — pre/code block styling acceptable
- [ ] Error states — Alert visible

#### Graph (`/graph`)
- [ ] Stats badges (`hlmBadge`) — visible and correctly placed
- [ ] Hover tooltip (`hlmCard`) — positioned correctly over canvas
- [ ] Canvas background color
- [ ] Error state — Alert visible
- [ ] Empty state — message readable (see P3)

#### Settings (`/settings`)
- [ ] Status cards (`hlmCard`) — engine info layout
- [ ] Badges (`hlmBadge`) — status indicators
- [ ] Dark mode toggle (`hlmSwitch`) — correct on/off state, label alignment
- [ ] Appearance card layout — Switch position
- [ ] Error state — Alert visible

#### Layout (global)
- [ ] Sidebar (`hlmSidebar`) — icon/collapsible behavior
- [ ] Footer dark mode Switch — visible and functional
- [ ] Top bar — hamburger trigger visible
- [ ] Toast notifications (ngx-sonner) — test after create/delete action

### Global Concerns

- [ ] **Dark mode** — all views look correct in both light and dark
- [ ] **Mobile responsive** — layout doesn't break below 768px
- [ ] **Keyboard navigation** — all interactive elements reachable via Tab
- [ ] **Focus indicators** — `focus-visible:ring` visible on all controls
- [ ] **Tailwind v4 classes** — no v3 leftover utility classes causing visual regressions

## Related Specs

- [Designer Review Follow-up — UI Polish](./designer-review-followup.md) — Prior design review fixes (sidebar SVGs, wm-card padding, theme classes). Not the implementation plan for this spec's P1–P3 findings.
- [Sim UI Designer Review — Results](./sim-ui-designer-review-results.md) — Completed pass/fail tracking with task backlog for P1/P2/N1–N3

## How to Review

**Completed 2026-07-20** using `ng serve` with mock-injector.js. For re-review or reference:

1. Build + run: `cd apps/wm-web && npm run tauri` (for Tauri) or `npx ng serve` (browser-only)
2. Navigate through each view, note any visual issues
3. Check dark mode toggle in sidebar footer
4. Test keyboard navigation (Tab through all interactive elements)
5. Resize browser to mobile width
6. Record results on the tracking page: `sim-ui-designer-review-results` — mark each item **pass** or **fail**
7. For fail items, create tasks with reproduction details
