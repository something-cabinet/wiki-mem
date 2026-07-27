---
{}
relates_to:
  - {type: references, target: wiki:concepts:web-ui-ux-principles}
  - {type: references, target: wiki:patterns:systematic-ux-audit-methodology}
---

id: wiki:decisions:replace-hardcoded-colors-with-css-variables

# Decision: Replace Hardcoded Colors with CSS Variable Theme Tokens

## Context

WM Web UI had hardcoded color values (`bg-red-50`, `text-red-500`, `bg-violet-50`, `bg-amber-50`, `bg-cyan-50`, `bg-orange-50`) across 6 views:

1. **Broke in dark mode** — hardcoded light colors appeared on dark backgrounds
2. **Were inconsistent** — errors used different patterns per view
3. **Had no semantic meaning** — colors chosen for visual variety, not meaning
4. **Made maintenance harder** — every view had its own color scheme

## Options Considered

1. **Keep hardcoded colors** — Minimal effort, dark mode stays broken
2. **Add dark-mode overrides** — Duplicates every color (Tailwind dark: variant)
3. **CSS variable theme tokens** — Use existing `--primary`, `--destructive`, `--muted` variables

## Decision

Option 3: **Replace all hardcoded colors with CSS variable-based theme tokens.**

## Rationale

- Project has comprehensive OKLCH theme system (light + dark mode)
- Theme tokens adapt automatically to active mode with zero code
- Semantic tokens (`bg-destructive/10`, `bg-success/10`) reinforce meaning through color
- Consistent error presentation improves Jakob's Law compliance

## Consequences

- **Positive:** All 6 views now properly support dark mode
- **Positive:** Consistent error pattern (`bg-destructive/10 border-destructive/20 rounded-lg`) across all views
- **Positive:** Tag colors in Memory view are now theme-aware
- **Neutral:** Some visual variety lost (violet/amber/cyan/orange were distinct) — semantic consistency wins
- **Documentation:** This pattern should be followed for all future UI work — never use hardcoded color values

## Related
- @wiki/tasks/d5cc21
- @wiki/concepts/web-ui-ux-principles
- @wiki/patterns/systematic-ux-audit-methodology