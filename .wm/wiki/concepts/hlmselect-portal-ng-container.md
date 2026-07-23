---
title: Failure: hlmSelect with ng-container crashes with NG0201 TemplateRef
type: concept
---

---
title: hlmSelect with ng-container crashes with NG0201 TemplateRef
type: concept
tags: [failure, angular, spartan-ui, select]
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

## Root cause
Three separate violations of Spartan UI select API:

1. `<ng-container hlmSelectPortal>` — `HlmSelectPortal` hosts `BrnPopoverContent` which requires a TemplateRef. `<ng-container>` doesn't provide one. Must use `*hlmSelectPortal` (structural directive with asterisk).

2. `<div brnSelect>` without `hlmSelect` — `BrnSelect` is only the state directive. The popover overlay comes from `HlmSelect` wrapper. Must use `<div hlmSelect>`.

3. Missing `BrnPopover` — fixed by using `hlmSelect` which includes it.

## Prevention
Always use this pattern:
```html
<div hlmSelect [value]="..." (valueChange)="...">
  <hlm-select-trigger>
    <hlm-select-value />
  </hlm-select-trigger>
  <hlm-select-content *hlmSelectPortal>
    <hlm-select-item value="...">Label</hlm-select-item>
  </hlm-select-content>
</div>
```

## Time lost
~1h debugging across 3 components

## Related
- @task:fix-settings-view--ng0201-templateref--connection-error-bugs
