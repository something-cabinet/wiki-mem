---
---

---
title: Failure: hlmSelect with ng-container crashes with NG0201 TemplateRef
type: concept
tags: [failure, angular, spartan-ui, select, positioning]
relates_to:
  - {type: references, target: wiki:concepts:proxy-stale-tool-list-failure}
  - {type: references, target: wiki:concepts:mcp-tool-unavailability-fallback}
  - {type: references, target: wiki:concepts:schema-error-tagged-enums}
  - {type: references, target: wiki:concepts:wm_page-tags-bug}
  - {type: references, target: wiki:concepts:missed-project-guidance-fjadra}
---

## What went wrong
All select components in the app were broken:
- Memory view crashed entirely (blank page)
- Page Edit dialog crashed on open
- Create Page dialog select silently did nothing
- Memory view dropdown positioned detached from trigger (added 2026-07-23)

## Root cause
Three separate violations of Spartan UI select API:

1. `<ng-container hlmSelectPortal>` — `HlmSelectPortal` hosts `BrnPopoverContent` which requires a TemplateRef. `<ng-container>` doesn't provide one. Must use `*hlmSelectPortal` (structural directive with asterisk).

2. `<div brnSelect>` without `hlmSelect` — `BrnSelect` is only the state directive. The popover overlay comes from `HlmSelect` wrapper. Must use `<div hlmSelect>`.

3. Missing `BrnPopover` — fixed by using `hlmSelect` which includes it.

### Additional positioning issue (discovered 2026-07-23)
Even when the structural pattern is correct, the dropdown can still render detached from its trigger if the host/trigger width chain is broken:

1. `<hlm-select-trigger>` is an Angular component host (custom element). Browsers default it to `display: inline` unless told otherwise. An inline wrapper around a block-level button creates an ambiguous box for CDK overlay origin calculation.
2. If the `<div hlmSelect>` host has no explicit width, the CDK popover anchor may derive a different width reference than the trigger, placing the dropdown in the wrong position.

**Fix:**
- Add `host: { class: 'block' }` to the `HlmSelectTrigger` component metadata so the trigger is always block-level
- Give the `<div hlmSelect>` host an explicit width class (e.g., `w-44 shrink-0`)
- Change the trigger from a fixed width (`w-44`) to `w-full` so it fills the host exactly

## Prevention
Always use this pattern:
```html
<div hlmSelect [value]="..." (valueChange)="..." class="w-44 shrink-0">
  <hlm-select-trigger class="w-full">
    <hlm-select-value placeholder="Select..." />
  </hlm-select-trigger>
  <hlm-select-content *hlmSelectPortal>
    <hlm-select-item value="...">Label</hlm-select-item>
  </hlm-select-content>
</div>
```

Key rules:
1. Always use `*hlmSelectPortal` (with asterisk) — never `<ng-container hlmSelectPortal>`
2. Always use `<div hlmSelect>` — not `<div brnSelect>`
3. Always constrain the host width (`class="w-44 shrink-0"`) and fill trigger to host (`class="w-full"` on trigger)
4. Ensure `HlmSelectTrigger` has `host: { class: 'block' }` in its component metadata

## Time lost
~1h debugging across 3 components (initial), additional ~30m on positioning fix

## Related
- @task:fix-settings-view--ng0201-templateref--connection-error-bugs