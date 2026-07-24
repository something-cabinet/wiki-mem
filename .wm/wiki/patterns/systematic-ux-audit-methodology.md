---
id: wiki:patterns:systematic-ux-audit-methodology
title: "Pattern: Systematic UX Audit Methodology"
type: pattern
tags: [ux, review, methodology, frontend]
status: draft
relates_to:
  - {type: references, target: wiki:decisions:replace-hardcoded-colors-with-css-variables}
  - {type: references, target: wiki:concepts:web-ui-ux-principles}
  - {type: references, target: wiki:tasks:web-ui-ux-audit-fix}
---
id: wiki:patterns:systematic-ux-audit-methodology

# Pattern: Systematic UX Audit Methodology

## Problem

How to systematically review and fix UI issues across an entire application? Ad-hoc reviews miss issues and don't ensure consistent quality across views.

## Solution

Use a **structured UX audit checklist** based on established UX laws and principles. Walk through every view/component systematically.

### Checklist Template

For **each** view/component, check:

| # | Check | Rationale |
|---|-------|-----------|
| 1 | **Empty state** | What does the user see on first load? |
| 2 | **Loading state** | Clear progress indicator? |
| 3 | **Error state** | Informative error? Recovery option? |
| 4 | **Edge cases** | No data, long data, invalid data handling |
| 5 | **Keyboard support** | Full keyboard navigation? |
| 6 | **Focus indicators** | `:focus-visible` rings on interactive elements |
| 7 | **Color usage** | Theme-aware CSS variables, no hardcoded colors |
| 8 | **Consistency** | Buttons/headers/cards match other views |
| 9 | **Affordance** | Clickable elements look clickable |
| 10 | **Spacing rhythm** | Consistent margins, padding, whitespace |

Across the **whole app**:

| # | Check | Rationale |
|---|-------|-----------|
| 11 | **Duplicated patterns** | 3+ copies of same pattern (inline spinners, error messages) |
| 12 | **Hardcoded values** | Colors, sizes, spacing that should be theme variables |
| 13 | **Accessibility** | aria-labels, reduced-motion, keyboard handlers, screen reader support |

### Application Process

1. **Read** every view file + global theme file
2. **Audit** — for each UX principle, note violations with exact line references
3. **Group** fixes by category (global, layout, per-view)
4. **Apply** all fixes systematically
5. **Verify** with build (`ng build`) + visual check

## When to Use

- Before releasing a new UI feature
- When onboarding to an existing UI codebase
- When dark mode support is needed
- When accessibility compliance is required
- During regular maintenance cycles

## When Not to Use

- For minor one-line UI tweaks
- When the app is purely functional/headless (no visual interface)
- When the team already has a mature design system with automated checks

## Related
- @wiki/decisions/replace-hardcoded-colors-with-css-variables
- @wiki/concepts/web-ui-ux-principles
- @wiki/tasks/d5cc21
