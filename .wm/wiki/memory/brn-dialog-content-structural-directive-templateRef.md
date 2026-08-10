---
title: BrnDialogContent (spartan-ng) must be used as structural directive with * prefix
type: memory
status: active
tags: [spartan-ng, angular, dialog, TemplateRef, NG0201]
---

## Problem

Using `brnDialogContent` (from `@spartan-ng/brain/dialog`) as a regular attribute directive:

```html
<div brnDialogContent class="...">
```

Causes: `NG0201: No provider for TemplateRef found`

## Root Cause

`BrnDialogContent` (parent of `BrnSheetContent`) unconditionally injects `TemplateRef`:

```javascript
class BrnDialogContent {
    _template = inject(TemplateRef);
    constructor() {
        this._brnDialog?.registerContent(this._template, this.context, this.className);
    }
}
```

`TemplateRef` is only provided by Angular when the directive is applied to an `<ng-template>` element — i.e., when used as a **structural directive** with the `*` prefix. Using it as a plain attribute directive (`div brnDialogContent`) skips the `<ng-template>` wrapper and `TemplateRef` is unavailable.

## Fix

Use the `*` prefix to make it a structural directive:

```html
<div *brnDialogContent class="...">
```

Angular desugars this to:
```html
<ng-template brnDialogContent>
  <div class="...">...</div>
</ng-template>
```

## Affected Directives

- `BrnDialogContent` — selector `[brnDialogContent]` from `@spartan-ng/brain/dialog`
- `BrnSheetContent` (extends `BrnDialogContent`) — selector `[brnSheetContent]` from `@spartan-ng/brain/sheet`
- Any directive using `BrnDialogContent` or `BrnSheetContent` as a host directive (e.g., `HlmSheetPortal` with `*hlmSheetPortal`)

## Detection

If you see `NG0201: No provider for TemplateRef found`, check if any spartan-ng brain directives with `*` syntax are being used without the `*` prefix. The selector `[brnDialogContent]` is a structural directive despite looking like an attribute selector.
