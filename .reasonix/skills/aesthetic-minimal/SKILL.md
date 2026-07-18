---
name: aesthetic-minimal
description: Minimal/Functional aesthetic — system fonts, restrained spacing, muted palette, no AI-slop look
---

# Aesthetic: Minimal / Functional

**Load this skill before any UI/design work.** It establishes the aesthetic direction and anti-patterns to avoid.

---

## Principles

- **Purpose over decoration** — every element earns its place. If it's not functional, remove it.
- **Low visual noise** — minimal borders, no gratuitous shadows or gradients, no floating decorative panels.
- **Consistent rhythm** — everything aligns to a spacing grid. Nothing is accidental.
- **High information density** (where appropriate) — tools and data views can be dense. Whitespace serves readability, not emptiness.
- **Accessibility first** — WCAG AA contrast minimum, no reliance on color alone for meaning.

## Typography

- **Family:** System UI stack only — `system-ui, -apple-system, sans-serif` (or the OS-native equivalent). **No Inter.** No custom typefaces for UI.
- **Monospace:** Only for code blocks, data values, or technical readouts — never for body text.
- **Scale:** Max 2 sizes per component: one heading weight, one body. No gratuitous hierarchy levels.
- **Weights:** 400 body, 600 heading. No bolder. No thinner.
- **Line height:** 1.5 body, 1.3 heading, 1.0 for data/monospace.
- **Uppercase:** sparingly — section labels, badges. Never for full sentences.

## Spacing & Grid

- **Base unit:** 4px (or 8px). Every spacing value must be a multiple of the base.
- **Container padding:** 16px minimum.
- **Element padding:** 8px minimum for interactive elements (buttons, inputs).
- **Gap between related elements:** 8-12px.
- **Gap between sections:** 24-32px.
- **Max content width:** 1200px (tool UIs), 720px (reading-focused pages).
- **No absolute positioning** for layout — flexbox or grid only.
- **Align everything to a consistent grid** — no element should be "close enough."

## Color

- **Palette:** Muted, restrained. Neutral grays for backgrounds and text.
  - Backgrounds: `#fff` or `#f5f5f5` (light); `#1a1a1a` or `#222` (dark).
  - Text: `#111` or `#333` (light); `#e0e0e0` or `#ccc` (dark).
  - Borders/separators: `#e0e0e0` (light); `#333` (dark).
- **Accent:** One accent color at most. Use it only for interactive elements (links, focus states, active indicators). Never for decorative fills.
- **No gradients** except data visualization (charts, progress bars).
- **No colored shadows.**
- **Contrast:** All text/icon combinations must pass WCAG AA (4.5:1 normal, 3:1 large).

## Interaction & States

- Hover: subtle background shift (`alpha: 0.05` overlay) or border color change. No scaling, no glow.
- Focus: clear 2px outline or ring — never `outline: none` without a replacement.
- Active/selected: distinct but restrained — a filled background, not a dramatic border or shadow.
- Disabled: 40% opacity. No color changes.
- Transitions: 150-200ms ease. No long or bouncy animations in UI chrome.

## Anti-Patterns (The "AI Slop" Look)

These are the patterns that make AI-generated UI look generic. **Reject them all.**

| ❌ Avoid | ✅ Instead |
|---|---|
| Soft gradients as page backgrounds | Flat neutral backgrounds |
| Floating decorative panels/cards | Purposeful containers with clear edges |
| Rounded corners > 8px | 4-8px radius max; 0px on tool UIs |
| Dramatic box shadows | Subtle border or 1px shadow at most |
| Inter font family | System UI stack |
| Purple + pastel accent combo | A single muted accent (blue, teal, or neutral) |
| Giant hero sections with illustration | Compact, content-first layout |
| Decorative icons with no function | Icons only when they aid scanning |
| Placeholder text echoing the label | `Placeholder` or empty state guidance |
| Instructions in UI copy ("Click here to...") | Direct, scannable labels |
| Broken/misaligned mobile layout | Responsive from the start |
| Orphaned empty states | Graceful empty states with clear action |

## Files Checklist (share this with the orchestrator)

Before delivering design output, verify:

- [ ] No hardcoded colors — all tokens from a CSS variable or token file
- [ ] All spacing multiples of the base unit
- [ ] Responsive at 320px (mobile), 768px (tablet), 1280px (desktop)
- [ ] No decorative-only elements
- [ ] No Inter font — system UI stack used
- [ ] WCAG AA contrast checked
- [ ] Hover, focus, active, disabled states defined for all interactive elements
- [ ] No placeholder/instruction text in UI copy
- [ ] Build compiles with zero errors
