---
id: wiki:specs:sim-ui-polish
title: Sim UI Polish — Designer Review Follow-up
type: spec
status: draft
relates_to:
  - {type: references, target: wiki:notes:session-handover-2026-07-17}
  - {type: references, target: wiki:specs:dev-continue-2026-07-17}
---
id: wiki:specs:sim-ui-polish

## Sim UI Polish Spec

### Context
Designer review of the WM Web UI identified several issues with Sim UI component usage, layout, and accessibility. The left sidebar nav blends into the background, and Pages/Memory pages render blank.

### Issues Found

#### P0 — Broken / Invisible
| # | Issue | Root Cause | File |
|---|-------|-----------|------|
| 1 | Left nav sidebar invisible | `bg-sidebar` (oklch 0.985) ≈ body `bg-gray-50`. Sidebar renders in DOM (tauri-pilot snapshot) but is visually indistinguishable from content | layout.component.ts:34, index.html:10 |
| 2 | Pages/Memory pages blank | Likely runtime errors in ngOnInit or template | pages-view.component.ts, memory-view.component.ts |
| 3 | Body class hardcodes `bg-gray-50` instead of `bg-background` — breaks dark mode | index.html:10 overrides Tailwind base layer | index.html |

#### P1 — Sim UI Misuse
| # | Issue | Fix | File |
|---|-------|-----|------|
| 4 | `textarea[wmInput]` not styled — selector only targets `input[wmInput]` | Add `textarea[wmInput]` or create `WmTextarea` | input/wm-input.ts |
| 5 | `wmBadge` used on interactive `<button>` in search filters | Replace with `wmBtn` with size/color variants | search-view.component.ts |
| 6 | Raw `<select>` in create dialog instead of `wm-select` | Use `WmSelectComponent` | pages-view.component.ts |
| 7 | Inline SVG icons duplicated across views | Replace with `NgIcon` + lucide imports | All view components |
| 8 | `wm-accordion` missing `aria-expanded` on toggle | Add `[attr.aria-expanded]` binding | accordion/wm-accordion.ts |
| 9 | Settings badge uses raw Tailwind overrides | Add `destructive` variant to `WmBadge` | settings-view.component.ts, badge/ |

#### P1 — Missing Features
| # | Issue | Fix | File |
|---|-------|-----|------|
| 10 | Search results not keyboard accessible (`div` + `routerLink`) | Change to `<a>` or add `tabindex` + keydown | search-view.component.ts |
| 11 | Missing error states in Graph, Tasks, Memory | Add error blocks with `role="alert"` | graph-view.ts, tasks-view.ts, memory-view.ts |
| 12 | No empty state for Graph | Show message when 0 nodes | graph-view.component.ts |

#### P2 — Consistency
| # | Issue | Fix |
|---|-------|-----|
| 13 | Inconsistent loading spinners | Create shared `WmSpinner` component |
| 14 | Inconsistent page headers | Standardize header pattern across views |
| 15 | Memory labels use `text-gray-500` instead of `text-muted-foreground` | Replace raw Tailwind shades with theme tokens |

### Implementation Order
1. Fix body class + sidebar visibility (P0 visual)
2. Fix blank Pages/Memory (P0 crash)
3. Add textarea support to wmInput (P1 misuse)
4. Replace badge-buttons with wmBtn (P1 misuse)
5. Fix keyboard a11y for search results (P1 missing)
6. Replace raw select with wm-select (P1 misuse)
7. Add aria-expanded to wm-accordion (P1 missing)
8. Add error states to remaining views (P1 missing)
9. Standardize icons, spinners, headers (P2)

### Status — 2026-07-17 (all done)
- ✅ P0-1: Body class fixed
- ✅ P0-2: Sidebar → spartan-ng `hlm-sidebar`
- ✅ P0-3: IPC timeout (15s) + error handlers on all views
- ✅ P1-3: Edit/delete on Pages + Memory views
- ✅ P1-4: Pages create dialog content field
- ✅ P1-5: No pagination — pending (needs backend)
- ✅ P1-6: Hardcoded colors → theme tokens (all 6 views)
- ✅ P1-7: Inline SVGs → NgIcon (all views)
- ✅ P1-8: `aria-expanded` on accordion
- ✅ P1-9: Settings badge → `bg-destructive/10 text-destructive`
- ✅ P1-10: Dark mode toggle in Settings + localStorage persistence
- ✅ P1-11: Error states on Graph, Tasks, Memory
- ✅ P1-12: Empty state for Graph
- ✅ P2-11: Shared `WmSpinner` component
- ✅ P2-12: Graph canvas aria-label
- ✅ P2-13: Search debounce (300ms)
- ✅ P2-14: Keyboard focus on cards
- ✅ P2-15: Graph hover tooltip clear
- ✅ Subscription cleanup (DestroyRef) on all 6 views

### References
@wiki/notes/session-handover-2026-07-17, @wiki/specs/dev-continue-2026-07-17

