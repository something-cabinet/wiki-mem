---
title: Web UI: focus trap, accessibility, colorblind, mobile, dark mode polish
type: task
status: done
tags: [review, web-ui, accessibility]
priority: medium
knowns_id: 5uep44
spec: specs/web-ui-polish-focus-trap-a11y-colorblind-mobile-dark-mode
---

# Web UI: focus trap, accessibility, colorblind, mobile, dark mode polish

> **Spec:** `specs/web-ui-polish-focus-trap-a11y-colorblind-mobile-dark-mode`

> *Imported from Knowns task `5uep44`*

# Web UI: focus trap, accessibility, colorblind, mobile, dark mode polish

## Description


Fix designer-review Web UI issues:

1. **Focus trap for dialogs** — Create `src/lib/actions/focusTrap.ts` Svelte action. Apply to ConfirmDialog, HelpOverlay, and modals. Trap Tab cycling inside dialog.

2. **Task card keyboard accessibility** (tasks/+page.svelte) — Add Space key handler alongside Enter. Add aria-label for screen readers.

3. **Blocked status cycle** (tasks/+page.svelte) — Add `blocked: 'todo'` to statusCycle. Add `blocked: 'blocked'` to statusDisplay.

4. **Colorblind-friendly indicators** (tasks/+page.svelte) — Add symbols (⚠●○) alongside color for priority. Add text indicators for column status headers.

5. **Mobile table overflow** (sources/+page.svelte) — Wrap in scrollable container. Add responsive card layout fallback below 640px.

6. **GraphView tooltip HTML injection** (GraphView.svelte) — Add escapeHtml() to sanitize node titles in vis-network tooltips.

7. **Focus ring visibility** (app.css) — Strengthen :focus and :focus-visible styles. Improve box-shadow opacity to 30%.

8. **Reduced motion support** (app.css) — Add `@media (prefers-reduced-motion: reduce)` to disable skeleton shimmer.

9. **Toast screen-reader support** (Toast.svelte) — Add `aria-live="polite"` to toast container.


## Acceptance Criteria



## Implementation Notes


Web UI polish implemented:
- focusTrap.ts Svelte action created, applied to ConfirmDialog and HelpOverlay
- Task cards: Space key handler, aria-label, colorblind symbols (⚠●○)
- Blocked status: cycle todo, display blocked
- Sources table: scrollable wrapper + responsive card layout at 640px
- GraphView: escapeHtml() for tooltips
- Focus rings: improved :focus-visible with 30% shadow
- Reduced motion: @media query disables skeleton shimmer
- Toast: aria-live="polite"
SvelteKit build passes.
