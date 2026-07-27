---
title: Sim UI Migration Patterns — Pitfalls & Process
type: howto
tags:
- learning
- angular
- sim-ui
- migration
status: draft
relates_to:
  - {type: references, target: wiki:specs:sim-ui-full-migration}
  - {type: references, target: wiki:specs:tauri-desktop-migration}
---

## Key Takeaways from the Sim UI Full Migration

### 1. Sim UI Components Are Spartan UI Helm Wrappers

Sim UI's `hlm-*` components are thin visual wrappers around `@spartan-ng/brain` directives. The pattern is:
- `BrnXxx` (brain) = headless behavior, accessibility
- `HlmXxx` (helm) = visual styling via `cva` + `classes()`

Understanding this split is critical — the `Brn*` imports come from `@spartan-ng/brain/xxx`, while `Hlm*` imports come from `@ui/xxx` (local copy).

### 2. Sim UI Uses Copy-Paste, Not npm Install

There's no `npm install sim-ui`. You copy the component source from simui.dev into your `src/libs/ui/` directory. This means:
- You own the code — can modify freely
- You must maintain imports manually
- Path aliases in `tsconfig.json` are essential (`@ui/button` etc.)

### 3. Common Pitfalls

| Pitfall | Symptom | Fix |
|---|---|---|
| Using `hlm-select-portal` as `<hlm-select-portal>` (element) | Build error: not a known element | Use `<ng-container hlmSelectPortal>` (it's a directive) |
| Using `hlm-option` (doesn't exist) | Build error: not a known element | Use `<hlm-select-item value="x">` (component) |
| Using `brn-select` as `<brn-select>` (element) | Build error: not a known element | Use `<div brnSelect>` (it's a directive) |
| Missing `hlm-select-portal` wrapper | Dropdown renders inline instead of overlaying | Wrap `<hlm-select-content>` in `<ng-container hlmSelectPortal>` |
| Accordion `[isOpened]="false"` hardcoded | Toggle breaks, column can never open | Initialize parent state variable instead |
| BrnSelect `value` input type | `string \| null \| undefined` error | Use `$event ?? ''` fallback in handler |
| Badge variant `"success"` | Type error — HlmBadge doesn't have it | Use `"secondary"` instead, or add a `--success` CSS variable |
| Theme tokens not registered in `@theme inline` | Tailwind v4 can't generate utility classes | Add `--color-success: var(--success)` etc. in `@theme inline` block |

### 4. Select Component Architecture

The Sim UI select has a specific hierarchy:

```
div[brnSelect]           ← container directive
  hlm-select-trigger     ← trigger button (component)
    hlm-select-value     ← selected value display (component)
  ng-container[hlmSelectPortal]  ← portal overlay (DIRECTIVE, not element)
    hlm-select-content   ← dropdown panel (component)
      hlm-select-item    ← option item (component, value input required)
```

### 5. Dialog Component Architecture

```
brn-dialog               ← container (component, [state]="open/closed")
  div[brnDialogOverlay][hlmDialogOverlay]  ← backdrop
  div[*brnDialogContent][hlmDialogContent] ← content panel
    hlm-dialog-header
      hlm-dialog-title
    hlm-dialog-footer
```

### 6. Tabs Component Architecture

```
div[hlmTabs][tab]="currentTab" (tabActivated)   ← container
  div[hlmTabsList]
    button[hlmTabsTrigger][hlmTabsTrigger]="tabId"  ← each tab
  div[hlmTabsContent][hlmTabsContent]="tabId"       ← content panel
```

### 7. Designer Review Process

Before fixing UI issues, get designer sign-off first:
1. Spawn `task` with `subagent_type: "designer"` for visual review
2. Document findings as P0-P3 in a spec
3. Only implement after designer confirms fixes

### 8. Wiki Tool Reliability

The `wm_page.*` tools have several bugs (see `wiki-tool-reliability-bugs.md` task). Workaround: write `.md` files directly and run `wm_index rebuild` afterward. The `wm_page.update` is particularly unreliable — it fails on "page not found" for pages that exist.