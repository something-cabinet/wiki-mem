---
title: Gray Areas — Theme Token Definition
type: spec
status: draft
tags: [spec, design-system, theme, css-variables]
---

## Overview

The UI uses CSS variable theme tokens for all surfaces, text, and borders. These render as various shades of gray in the light theme. This spec defines each "gray area" token — what it represents, where it's used, and whether the current usage is correct.

## Theme Tokens (Light Values)

| Token | Value | Renders As | Purpose |
|---|---|---|---|
| `--background` | `oklch(1 0 0)` | White | Page background |
| `--foreground` | `oklch(0.145 0 0)` | Near-black | Primary text |
| `--card` | `oklch(1 0 0)` | White | Card/panel surfaces |
| `--card-foreground` | `oklch(0.145 0 0)` | Near-black | Card text |
| `--popover` | `oklch(1 0 0)` | White | Dropdown/popover surface |
| `--popover-foreground` | `oklch(0.145 0 0)` | Near-black | Popover text |
| `--primary` | `oklch(0.205 0 0)` | Near-black | Primary buttons/accents |
| `--secondary` | `oklch(0.97 0 0)` | Light gray | Secondary surfaces |
| `--secondary-foreground` | `oklch(0.205 0 0)` | Near-black | Secondary text |
| `--muted` | `oklch(0.97 0 0)` | Light gray | Muted backgrounds |
| `--muted-foreground` | `oklch(0.556 0 0)` | Medium gray | Secondary/label text |
| `--accent` | `oklch(0.97 0 0)` | Light gray | Hover/accent surfaces |
| `--accent-foreground` | `oklch(0.205 0 0)` | Near-black | Accent text |
| `--destructive` | `oklch(0.577 0.245 27.325)` | Red | Destructive/delete |
| `--border` | `oklch(0.922 0 0)` | Light gray border | Default borders |
| `--input` | `oklch(0.922 0 0)` | Light gray border | Input borders |
| `--ring` | `oklch(0.708 0 0)` | Medium gray | Focus rings |
| `--sidebar` | *(from spartan preset)* | — | Sidebar surface |
| `--sidebar-foreground` | *(from spartan preset)* | — | Sidebar text |
| `--sidebar-accent` | *(from spartan preset)* | — | Sidebar hover |
| `--sidebar-border` | *(from spartan preset)* | — | Sidebar borders |
| `--success` | *(not a standard token)* | Green-ish | Success states (used in tasks) |

## Gray Area Usages by View

### Layout (sidebar + main wrapper)

| Element | Current Classes | Token | Issue |
|---|---|---|---|
| Main content wrapper | `bg-muted/20` | `--muted` at 20% | Very light gray, ok |
| Top header bar | `bg-background border-b border-border` | `--background`, `--border` | White bg with gray bottom border, ok |
| Sidebar header | `border-b border-sidebar-border` | `--sidebar-border` | Sidebar divider, ok |
| Sidebar footer | `border-t border-sidebar-border` | `--sidebar-border` | Same pattern, ok |
| Sidebar version text | `text-sidebar-foreground/60` | `--sidebar-foreground` at 60% | Low emphasis, ok |
| Sidebar dark mode button | `hover:bg-sidebar-accent` | `--sidebar-accent` | Hover state, ok |

### Search

| Element | Current Classes | Token | Issue |
|---|---|---|---|
| Header | `bg-card border-b border-border` | `--card`, `--border` | White bg, ok |
| Search icon | `text-muted-foreground` | `--muted-foreground` | Medium gray icon, ok |
| Type label | `text-xs text-muted-foreground uppercase` | `--muted-foreground` | Label color, ok |
| Loading text | `text-muted-foreground` | `--muted-foreground` | Status text, ok |
| Score text | `text-xs text-muted-foreground font-mono` | `--muted-foreground` | Secondary info, ok |
| Empty state icon | `text-muted-foreground/30` | 30% opacity | Very faint, ok |
| Empty state text | `text-muted-foreground` | `--muted-foreground` | Guide text, ok |
| Enter kbd | `bg-muted` | `--muted` | Keyboard hint bg, ok |
| No results text | `text-muted-foreground` | `--muted-foreground` | Empty result, ok |

### Tasks

| Element | Current Classes | Token | Issue |
|---|---|---|---|
| Header | `bg-card border-b border-border` | `--card`, `--border` | White bg, ok |
| Accordion | `border border-border` | `--border` | Column outline, ok |
| Accordion content | `bg-muted/20` | 20% muted | Inner panel, ok |
| Task ID | `text-xs text-muted-foreground` | `--muted-foreground` | Secondary info, ok |
| Empty column | `text-muted-foreground/60` | 60% opacity | Very low emphasis, ok |
| Loading | `text-muted-foreground` | `--muted-foreground` | Status text, ok |
| Column header (todo) | `bg-muted/40 text-muted-foreground hover:bg-muted/60` | `--muted` at 40% | Neutral column header |
| Column header (in-review) | `bg-accent/10 text-accent-foreground` | `--accent` at 10% | Review column |
| Column header (done) | `bg-success/10 text-success` | **`--success` not defined** | Uses custom `success` |
| Column header (blocked) | `bg-destructive/10 text-destructive` | `--destructive` at 10% | Blocked column |
| Column dot (todo) | `bg-muted-foreground/40` | 40% opacity | Low emphasis dot |
| Column dot (done) | `bg-success` | **`--success`** | Needs definition |
| Column dot (blocked) | `bg-destructive` | `--destructive` | Red dot, ok |

### Memory

| Element | Current Classes | Token | Issue |
|---|---|---|---|
| Header | `bg-card border-b border-border` | `--card`, `--border` | White bg, ok |
| Labels | `text-muted-foreground uppercase tracking-wider` | `--muted-foreground` | Form labels, ok |
| Dialog text | `text-muted-foreground` | `--muted-foreground` | Confirm text, ok |
| Date | `text-xs text-muted-foreground font-mono` | `--muted-foreground` | Timestamp, ok |
| Entry content | `text-sm text-muted-foreground` | `--muted-foreground` | Body text, ok |
| Empty state | `text-muted-foreground text-center` | `--muted-foreground` | Empty message, ok |
| Edit button hover | `hover:text-red-500` | **`--destructive` not `red-500`** | Inconsistent hover |
| Loading | `text-muted-foreground` | `--muted-foreground` | Status text, ok |

### Pages

| Element | Current Classes | Token | Issue |
|---|---|---|---|
| Header | `bg-card border-b border-border` | `--card`, `--border` | White bg, ok |
| Status badge | `bg-muted/50 text-muted-foreground` | 50% muted | Status indicator, ok |
| Code block | `bg-muted/30 border border-border` | 30% muted | Content display, ok |
| Labels | `text-muted-foreground uppercase tracking-wider` | `--muted-foreground` | Form labels, ok |
| Page ID | `text-xs text-muted-foreground font-mono` | `--muted-foreground` | Secondary info, ok |
| Loading | `text-muted-foreground` | `--muted-foreground` | Status text, ok |
| Delete text | `text-sm text-muted-foreground` | `--muted-foreground` | Confirm dialog, ok |

### Graph

| Element | Current Classes | Token | Issue |
|---|---|---|---|
| Header | `bg-card border-b border-border` | `--card`, `--border` | White bg, ok |
| Canvas area | `bg-muted/30` | 30% muted | Muted canvas bg, ok |
| Stats | `text-sm text-muted-foreground` | `--muted-foreground` | Node/edge counts, ok |
| Spacing label | `text-xs text-muted-foreground` | `--muted-foreground` | Slider label, ok |
| Spacing value | `text-xs text-muted-foreground` | `--muted-foreground` | Slider value, ok |
| Loading | `text-muted-foreground` | `--muted-foreground` | Status text, ok |
| Tooltip ID | `text-muted-foreground font-mono` | `--muted-foreground` | Node ID, ok |
| Tooltip degree | `text-muted-foreground` | `--muted-foreground` | Edge count, ok |
| Empty state | `text-muted-foreground font-medium` | `--muted-foreground` | Empty message, ok |
| Empty guide | `text-xs text-muted-foreground/60` | 60% opacity | Guide text, ok |

### Settings

| Element | Current Classes | Token | Issue |
|---|---|---|---|
| Header | `bg-card border-b border-border` | `--card`, `--border` | White bg, ok |
| Section titles | `text-muted-foreground uppercase tracking-wider` | `--muted-foreground` | Section labels, ok |
| Property labels | `text-muted-foreground` | `--muted-foreground` | DL dt elements, ok |
| Loading | `text-muted-foreground` | `--muted-foreground` | Status text, ok |
| Stale badge | `bg-destructive/10 text-destructive` | `--destructive` | Error badge, ok |

## Issues Found

### P1 — `--success` Token Not Defined

Used in Tasks view for the "done" column header and dot indicator:
- `bg-success/10 text-success` — column header
- `bg-success` — status dot

**Fix:** Add `--success` as an oklch green tone to `:root` in `styles.css`. Suggested value: `oklch(0.627 0.194 149.214)` (green).

### P2 — `hover:text-red-500` Hardcoded in Memory View

**File:** `memory-view.component.ts` line 186
```html
<button hlmBtn variant="ghost" size="sm" (click)="startDelete(e)" class="text-muted-foreground hover:text-red-500">
```

`text-red-500` bypasses the theme token system. Should use `hover:text-destructive` instead.

**Fix:** Replace `hover:text-red-500` with `hover:text-destructive`.

### P3 — Inconsistent Muted Opacity Levels

| Opacity | Used In | Count |
|---|---|---|
| `bg-muted/20` | Layout main bg, accordion content | 2 |
| `bg-muted/30` | Graph canvas, code blocks | 2 |
| `bg-muted/40` | Task column header (todo default) | 1 |
| `bg-muted/50` | Status badge (Pages) | 1 |
| `bg-muted` (full) | Search kbd | 1 |

Currently there's no standard — each view uses slightly different opacity. Worth standardizing to reduce visual randomness.

### P3 — Repeating Header Pattern

All 6 views use identical header:
```html
<header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
```

This is consistent (good) but could be extracted into a shared component to avoid repetition (future concern, not blocking).

## Recommended Standardization

| Purpose | Token | Opacity |
|---|---|---|
| Page content background | `bg-muted` | `/20` |
| Panel inside content (accordion, code) | `bg-muted` | `/30` |
| Hover state on muted surface | `bg-accent` | `/50` |
| Status indicator | `bg-muted` | `/50` |
| Secondary labels | `text-muted-foreground` | full |
| Low emphasis text | `text-muted-foreground` | `/60` |
| Empty state text | `text-muted-foreground` | full |
| Header surface | `bg-card` | full |
| Header bottom border | `border-border` | full |
