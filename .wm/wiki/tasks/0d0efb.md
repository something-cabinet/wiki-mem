---
title: Refactor Pages dialog/URL state management
id: 0d0efb
type: task
status: done
priority: high
tags: [bug, web-ui, pages, ux]
acceptance_criteria:
  - text: "Edit/Delete dialogs are moved to the template root so view state is preserved until the user confirms"
  - text: "loadPage() calls router.navigate so the URL stays in sync with the currently viewed page"
  - text: "ngOnInit subscribes to paramMap so navigating to a different page while already viewing one reloads the view"
---

From @designer review C6: (1) Edit/Delete dialogs trapped inside @else branch — opening them destroys view state before user confirms. (2) loadPage() never calls router.navigate — URL doesn't sync. (3) ngOnInit reads route.snapshot once — navigating to different page while already viewing one does nothing. Fix: move dialogs to template root, make loadPage navigate, subscribe to paramMap.

## Implementation notes (done 2026-08-08)
- AC1 (dialogs at template root) is **MOOT**: the edit/delete write UI was removed by design in commit c0739a6 — no `hlm-dialog` (or any dialog) exists anywhere in the views (`rg Dialog apps/wm-web/src/app/views` returns nothing). The pages view is read-only (list + content view), so there is no dialog state to preserve.
- AC2 done: `openPage()` calls `router.navigate(['/pages', id])` (pages-view.component.ts:177), keeping the URL in sync.
- AC3 done: `ngOnInit` subscribes to `route.paramMap` with `takeUntilDestroyed`, so navigating to a different page while already viewing one reloads the view (pages-view.component.ts:118–130).
- Verified no leftover dialog imports or remnants; `tsc --noEmit` and `ng build` pass.
