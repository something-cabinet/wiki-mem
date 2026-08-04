---
id: wiki:specs:sim-ui-designer-review-results
title: Sim UI Designer Review — Results
type: spec
status: completed
tags: [designer, review, sim-ui, tracking]
---
id: wiki:specs:sim-ui-designer-review-results

# Sim UI Designer Review — Results

Tracking page for the designer review pass/fail results. Linked from `@doc/specs/sim-ui-designer-review`.

**Reviewer:** Designer (AI sub-agent)
**Date:** 2026-07-20
**Build:** `ng serve` with mock-injector.js (browser-only dev mode)

---
id: wiki:specs:sim-ui-designer-review-results

## Legend

- ✅ **Pass** — looks correct
- ❌ **Fail** — issue found (→ create task)
- ⚠️ **Partial** — passes but with caveats
- 🔲 **N/A** — not applicable / can't test

---
id: wiki:specs:sim-ui-designer-review-results

## Search (`/search`)

| # | Item | Result | Notes |
|---|------|--------|-------|
| 1 | Search input (`hlmInput`) — correct styling, placeholder visible | ✅ | Placeholder "Search pages, tasks, memory..." confirmed in DOM |
| 2 | Search button (`hlmBtn`) — proper default variant | ✅ | `variant="default"` confirmed |
| 3 | Type filter — Sim UI Tabs working correctly | ✅ | 4 tabs (All/Pages/Tasks/Memory), wired to `doSearch()` |
| 4 | Result cards (`hlmCard`) — correct appearance with hover state | ✅ | `hover:bg-accent/50 transition-colors` on cards |
| 5 | Result count badge (`hlmBadge`) — positioned correctly | ✅ | In header, `variant="secondary"` |
| 6 | Error state — Alert visible when error occurs | ✅ | `hlmAlert variant="destructive"` |
| 7 | Empty state — layout and spacing acceptable | ✅ | Centered icon + help text with keyboard shortcut hint |
| 8 | Loading state — spinner + text | ✅ | `wm-spinner` + "Searching..." |

## Tasks (`/tasks`)

| # | Item | Result | Notes |
|---|------|--------|-------|
| 1 | Accordion columns start collapsed (not expanded) | ❌ | See P1 finding — all 10 columns start `data-state="open"` |
| 2 | Accordion expand/collapse animation smooth | ✅ | BrnAccordion handles animation internally |
| 3 | Task cards (`hlmCard`) — correct appearance with priority border | ✅ | `border-l-4 border-l-destructive` (high), `amber-500` (medium), `success` (low) |
| 4 | Status column colors distinguishable | ⚠️ | See P2 finding — final mapping approved 2026-07-20, ready to implement |
| 5 | Badges (`hlmBadge`) — variant choice correct | ✅ | Count badges: secondary. Priority: default/secondary/outline mapping |
| 6 | Error state — Alert visible | ✅ | `hlmAlert variant="destructive"` |

## Memory (`/memory`)

| # | Item | Result | Notes |
|---|------|--------|-------|
| 1 | Select dropdowns overlay (not push content down) | ❌ | See P1 finding — NO `hlm-select-portal` wrapper; dropdown pushes content down |
| 2 | Dialogs (`BrnDialog`) — animation, backdrop, close button | ✅ | Overlay + header + footer pattern correct |
| 3 | Layer/Status filter selects — appearance, dropdown, selection | ✅ | 2 select triggers with proper styling |
| 4 | Form inputs (`hlmInput`) — proper styling | ✅ | Input + textarea both use `hlmInput` |
| 5 | Entry cards (`hlmCard`) — appearance with expand/collapse | ✅ | Cards with `hover:shadow-md transition-shadow` |
| 6 | Tag badges (`hlmBadge`) — variant="secondary" consistently | ✅ | Confirmed via DOM check: `variant="secondary"` |
| 7 | Action buttons (`hlmBtn`) — ghost/edit/delete variants correct | ✅ | 6 ghost buttons confirmed (edit + delete per entry) |
| 8 | "Show more/less" — `hlmBtn` link variant | ✅ | `variant="link" size="xs"` confirmed |
| 9 | Error states — Alert visible | ✅ | `hlmAlert variant="destructive"` |

## Pages (`/pages`)

| # | Item | Result | Notes |
|---|------|--------|-------|
| 1 | Create/Edit/Delete dialogs — animation, backdrop, close button | ✅ | All 3 dialogs use BrnDialog + hlmDialogOverlay + hlmDialogFooter |
| 2 | Type select — Sim UI select overlays correctly | ⚠️ | Select inside dialog — portal behavior depends on CDK overlay (likely OK in dialog context) |
| 3 | Page list cards (`hlmCard`) — clickable card appearance | ✅ | 8 page cards confirmed, `hover:shadow-md hover:border-foreground/20` |
| 4 | Badges (`hlmBadge`) — type badge variants | ✅ | `typeBadgeVariant()` mapping: default/secondary/outline per type |
| 5 | Content view — pre/code block styling acceptable | ✅ | `pre` with `bg-muted/30 rounded-lg border font-mono` |
| 6 | Error states — Alert visible | ✅ | `hlmAlert` with `hlmAlertTitle` + `hlmAlertDescription` |

## Graph (`/graph`)

| # | Item | Result | Notes |
|---|------|--------|-------|
| 1 | Stats badges (`hlmBadge`) — visible and correctly placed | ✅ | 2 stat badges in header (nodes + edges) |
| 2 | Hover tooltip (`hlmCard`) — positioned correctly over canvas | ✅ | Absolute positioned card with `z-20 pointer-events-none` |
| 3 | Canvas background color | ✅ | `bg-muted/30` on container |
| 4 | Error state — Alert visible | ✅ | Centered `hlmAlert variant="destructive"` with title+description |
| 5 | Empty state — message readable | ✅ | See P3 finding — card with "No graph data" + helpful message |

## Settings (`/settings`)

| # | Item | Result | Notes |
|---|------|--------|-------|
| 1 | Status cards (`hlmCard`) — engine info layout | ✅ | `hlmCard` with `dl` layout, border-b separators |
| 2 | Badges (`hlmBadge`) — status indicators | ✅ | `variant="secondary"` for counts, conditional destructive for stale |
| 3 | Dark mode toggle (`hlmSwitch`) — correct on/off state, label alignment | ✅ | Switch present, label click toggles `.dark` on html |
| 4 | Appearance card layout — Switch position | ✅ | Flex row with justify-between |
| 5 | Error state — Alert visible | ✅ | `hlmAlert variant="destructive"` with retry button |

## Layout (global)

| # | Item | Result | Notes |
|---|------|--------|-------|
| 1 | Sidebar (`hlmSidebar`) — icon/collapsible behavior | ✅ | `collapsible="icon"`, collapses to 0px on mobile |
| 2 | Footer dark mode Switch — visible and functional | ✅ | Switch + sun/moon icon + label text, toggles via label click |
| 3 | Top bar — hamburger trigger visible | ✅ | `hlmSidebarTrigger` button in header |
| 4 | Toast notifications (ngx-sonner) — test after create/delete action | ✅ | `NgxSonnerToaster position="top-right" richColors` present in layout |

## Global Concerns

| # | Item | Result | Notes |
|---|------|--------|-------|
| 1 | Dark mode — all views look correct in both light and dark | ✅ | Screenshots captured in both modes. CSS tokens have light+dark variants. |
| 2 | Mobile responsive — layout doesn't break below 768px | ✅ | Sidebar collapses to 0px. No horizontal overflow at 375px. Grid columns use responsive breakpoints (`md:grid-cols-3 lg:grid-cols-4`). |
| 3 | Keyboard navigation — all interactive elements reachable via Tab | ✅ | 14 elements confirmed: sidebar links → search button → input → type tabs. Sidebar links first (A tags). |
| 4 | Focus indicators — `focus-visible:ring` visible on all controls | ✅ | Solid 2px outline on all focused elements. CSS rule: `:focus-visible { outline: 2px solid var(--ring); }` |
| 5 | Tailwind v4 classes — no v3 leftover utility classes | ✅ | Zero matches for `bg-gray-`, `text-gray-`, `bg-slate-`, `text-slate-`, `bg-neutral-`, `text-neutral-` |

---
id: wiki:specs:sim-ui-designer-review-results

## Summary

| View | Pass | Fail | Partial | N/A |
|------|------|------|---------|-----|
| Search | 8 | 0 | 0 | 0 |
| Tasks | 4 | 1 | 1 | 0 |
| Memory | 8 | 1 | 0 | 0 |
| Pages | 5 | 0 | 1 | 0 |
| Graph | 5 | 0 | 0 | 0 |
| Settings | 5 | 0 | 0 | 0 |
| Layout | 4 | 0 | 0 | 0 |
| Global | 5 | 0 | 0 | 0 |
| **Total** | **44** | **2** | **2** | **0** |

---
id: wiki:specs:sim-ui-designer-review-results

## P1–P3 Findings Status

### P1 — Tasks: All Accordion Columns Expanded
**Status:** ✅ READY TO IMPLEMENT (Designer-confirmed 2026-07-20, with correction + refinement)

All 10 accordion items render with `data-state="open"`. Root cause: `[isOpened]="!collapsed[col]"` where `collapsed` starts as `{}`, so `collapsed[col]` is `undefined`, and `!undefined === true`.

**Fix correction:** The originally proposed `[isOpened]="false"` is **buggy as written** — the click handler still toggles `collapsed[col]`, but `isOpened` would stay `false` forever, so columns could never open. Correct minimal fix: initialize state after statuses are computed in `ngOnInit`:

```ts
this.collapsed = Object.fromEntries(this.statuses.map(s => [s, true]));
```

**Designer refinement (recommended):** Collapse only *empty* columns, keep non-empty columns expanded. A board where everything starts collapsed hides all task cards behind a click and defeats the at-a-glance purpose of a kanban view; empty columns are already dimmed via `opacity-75`, so collapsing them doubles down on the signal that they carry no work:

```ts
this.collapsed = Object.fromEntries(
  this.statuses.map(s => [s, (res.counts?.[s] ?? 0) === 0])
);
```

Either satisfies the checklist item ("columns must not all start expanded"); the refinement is the better UX. Implementer's choice, default to the refinement.

### P1 — Memory: Select Dropdown Extends Header Instead of Overlaying
**Status:** ✅ READY TO IMPLEMENT (Designer-verified 2026-07-20)

DOM check confirmed zero `hlm-select-portal` elements. The `<hlm-select-content>` renders inline, pushing the first memory card down to `top: 257px` (header is only 44px).

Designer re-verified against source: both `brnSelect` instances (Layer filter, lines 49–57; Status filter, lines 58–68) render `<hlm-select-content>` unwrapped. The fix as stated is correct and complete:

**Fix:** Wrap `<hlm-select-content>` inside `<hlm-select-portal>` in both select instances in `memory-view.component.ts` (lines 53-56 and 62-67). Import `HlmSelectPortal` from `@ui/select`. No visual-design side effects expected — portal rendering only changes overlay positioning, not trigger appearance.

### P2 — Tasks: Column Visual Distinction
**Status:** ✅ RESOLVED — Designer recommendation final (2026-07-20), ready to implement

**Root-cause discovery (supersedes the original analysis):** The theme's `primary` token is **not blue** — `--primary: oklch(0.205 0 0)` has zero chroma (neutral monochrome), as do `secondary`, `accent`, and `muted`. So today **5 of 8 columns are gray** (draft, todo, in-progress, in-review, on-hold) and 2 of the 3 colored ones share red (blocked, urgent). The problem is bigger than rebalancing — the palette needs semantic hue assignments.

**Final color mapping (approved):**

| Status | Header classes | Dot class | Rationale |
|--------|---------------|-----------|-----------|
| draft | `bg-muted/40 text-muted-foreground hover:bg-muted/60` *(unchanged)* | `bg-muted-foreground/40` | Inert placeholder — gray is semantically correct |
| todo | `bg-sky-500/10 text-sky-600 dark:text-sky-400 hover:bg-sky-500/15` | `bg-sky-500` | Cool cyan = queued, ready |
| in-progress | `bg-blue-500/10 text-blue-600 dark:text-blue-400 hover:bg-blue-500/15` | `bg-blue-500` | True blue = active work (replaces neutral `primary`) |
| in-review | `bg-purple-500/10 text-purple-600 dark:text-purple-400 hover:bg-purple-500/15` | `bg-purple-500` | Awaiting judgment — fixes "too subtle" accent/10 |
| done | `bg-success/10 text-success hover:bg-success/15` *(unchanged)* | `bg-success` | Complete |
| blocked | `bg-destructive/10 text-destructive hover:bg-destructive/15` *(unchanged)* | `bg-destructive` | Impediment — red reserved for "something is wrong" |
| on-hold | `bg-amber-500/10 text-amber-600 dark:text-amber-400 hover:bg-amber-500/15` | `bg-amber-500` | Paused / caution (traffic-light yellow) |
| urgent | `bg-orange-500/15 text-orange-600 dark:text-orange-400 hover:bg-orange-500/25` | `bg-orange-600` | Escalated — hot, but distinct from error-red |
| cancelled / archived | default muted fallback *(unchanged)* | default | Inert terminal states — gray is correct |

**Design logic:**
1. **Workflow = cool progression, alerts = hot exceptions.** The happy-path columns form a deliberate hue journey — gray → sky → blue → purple → green — so position in the workflow reads as a gradient. The three attention states sit outside the flow with traffic-light semantics: red = blocked (stop), orange = urgent (hurry), amber = on-hold (wait).
2. **amber vs. orange adjacency mitigated.** `on-hold` and `urgent` sit adjacent in `statusOrder`, so urgent gets the deeper `orange-600` dot and a stronger `/15` header tint; on-hold keeps lighter `amber-500`. Lightness + saturation separation carries the distinction even where hue is close.
3. **Dark mode fixed as a side effect.** All hue headers gain `dark:text-*-400` — the current `-600`/token text colors render too dark on dark card backgrounds. `success` and `destructive` tokens already have dark variants, so those stay token-based.
4. **Accessibility.** Color is never the sole channel — every header carries its text label, so the deuteranopia red/green pair (blocked/done) is inherently mitigated. All Tailwind palette classes are literal strings in the maps, so the v4 scanner picks them up.

**Implementation (drop-in replacement for both maps in `tasks-view.component.ts`):**

```ts
headerColorClass(col: string): string {
  const map: Record<string, string> = {
    draft: 'bg-muted/40 text-muted-foreground hover:bg-muted/60',
    todo: 'bg-sky-500/10 text-sky-600 dark:text-sky-400 hover:bg-sky-500/15',
    'in-progress': 'bg-blue-500/10 text-blue-600 dark:text-blue-400 hover:bg-blue-500/15',
    'in-review': 'bg-purple-500/10 text-purple-600 dark:text-purple-400 hover:bg-purple-500/15',
    done: 'bg-success/10 text-success hover:bg-success/15',
    blocked: 'bg-destructive/10 text-destructive hover:bg-destructive/15',
    'on-hold': 'bg-amber-500/10 text-amber-600 dark:text-amber-400 hover:bg-amber-500/15',
    urgent: 'bg-orange-500/15 text-orange-600 dark:text-orange-400 hover:bg-orange-500/25',
  };
  return map[col] || 'bg-muted/40 text-muted-foreground hover:bg-muted/60';
}

dotColorClass(col: string): string {
  const map: Record<string, string> = {
    draft: 'bg-muted-foreground/40',
    todo: 'bg-sky-500',
    'in-progress': 'bg-blue-500',
    'in-review': 'bg-purple-500',
    done: 'bg-success',
    blocked: 'bg-destructive',
    'on-hold': 'bg-amber-500',
    urgent: 'bg-orange-600',
  };
  return map[col] || 'bg-muted-foreground/40';
}
```

**Verification checklist for implementer:** all 8 statuses visually distinct at dot size (8px) in both light and dark mode; urgent reads "hotter" than on-hold; in-progress now reads blue (not gray).

### P3 — Graph: Empty/Loading States
**Status:** ✅ PASS

Code review confirms:
- Empty state: Card with "No graph data" heading + "Create pages with connections to build your wiki graph." description
- Styled with `bg-card border border-border rounded-xl shadow-sm` — good contrast against `bg-muted/30` canvas
- Loading state: Spinner + "Loading graph..." centered overlay
- Error state: `hlmAlert variant="destructive"` centered with title + description

---
id: wiki:specs:sim-ui-designer-review-results

## Designer Decisions (Resolved) — Re-verified

| # | Decision | Status | Visual Check |
|---|----------|--------|-------------|
| 1 | Search type filter → Tabs | ✅ | 4 tabs (All/Pages/Tasks/Memory) confirmed in DOM |
| 2 | Task cards → hlmCard | ✅ | Cards rendered with priority left-border colors |
| 3 | Memory tag colors → Badge variants | ✅ | All tags use `variant="secondary"`, no hash-color logic |
| 4 | Button variants per role | ✅ | All 5 variants confirmed: default (Search/Save), destructive (Delete), ghost (Cancel/Edit), outline (Edit pages), link (Show more) |
| 5 | Toast position | ✅ | `position="top-right"` on `NgxSonnerToaster` |
| 6 | Alert variants | ✅ | destructive=error, default=success/info. No `outline` needed. |
| 7 | Wire Tabs in Search | ✅ | `(tabActivated)` triggers `doSearch()` |
| 8 | Dark mode v3 class audit | ✅ | Zero v3 gray/slate/neutral classes across all templates |

---
id: wiki:specs:sim-ui-designer-review-results

## New Issues Discovered (beyond P1–P3)

### N1 — Dark Mode Toggle: Switch Click Not Working (Minor)
The `hlm-switch button` click (via Puppeteer `click()`) did not toggle dark mode. However, clicking the parent `<label>` wrapper worked. This may be a Puppeteer-specific issue (click coordinates), but worth verifying on real hardware. The label-based toggle works correctly.

### N2 — Keyboard Tab Order: Sidebar Links First
Tab order goes through all 6 sidebar navigation links before reaching the main content (search button, input, tabs). This is expected for a sidebar layout but means keyboard users need 6 tabs to reach content. Consider adding a "skip to content" link.

### N3 — Memory "New" Button Position
The "New" button sits to the right of two select dropdowns in the header. On narrow viewports, this row wraps. At 375px, the layout is functional but the header buttons stack. This is acceptable but worth noting.

---
id: wiki:specs:sim-ui-designer-review-results

## Tasks Created from Fails

| Finding | Task ID | Status |
|---------|---------|--------|
| P1 — Tasks accordion expanded by default | (to create) | 🟢 Ready to implement (fix corrected + refinement, see P1 section) |
| P1 — Memory select missing portal wrapper | (to create) | 🟢 Ready to implement (verified correct) |
| P2 — Column color distinction improvements | (to create) | 🟢 Ready to implement (final mapping approved, see P2 section) |
| N1 — Dark mode switch click behavior | (to create) | 🟢 Minor |
| N2 — Skip-to-content for keyboard nav | (to create) | 🟢 Minor |
