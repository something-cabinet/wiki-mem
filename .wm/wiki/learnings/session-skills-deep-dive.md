---
title: Session Handover 2026-07-17
type: howto
tags: [handover, session, architecture, tauri, webgl]
---

# Critical Learnings from Session

## 1. Angular 22 PostCSS Config Must Be JSON

Angular's `@angular/build:application` builder **only reads `postcss.config.json`**, not `.js` or `.mjs`. A `postcss.config.js` with ESM `export default` is silently ignored, causing Tailwind v4 to stop processing after any HMR rebuild. The UI appears to "disappear" because all theme utilities (`bg-background`, `text-foreground`, `border-border`) become invalid CSS.

**Fix:** Use `postcss.config.json`:
```json
{ "plugins": { "@tailwindcss/postcss": {}, "postcss-nesting": {} } }
```

## 2. Tauri Crate-Type Conflict with Proc Macros

When adding Tauri v2 to a workspace that uses `turso` (or any crate with proc macros), the default `crate-type = ["staticlib", "cdylib", "rlib"]` causes proc macro DLL linking failures on Windows. Removing `cdylib` fixes it:
```toml
crate-type = ["staticlib", "rlib"]
```

## 3. regl v2 Has Built-in Types

`@types/regl` doesn't exist on npm. regl v2 ships its own TypeScript definitions at `dist/regl.d.ts`. Import directly:
```typescript
import REGL from 'regl';
```

## 4. Canvas 2D + WebGL Dual Renderer Pattern

For graph visualization, keep both renderers and switch at a threshold:
- Canvas 2D for <500 nodes (simpler, interactive, d3-force works)
- WebGL for 500+ nodes (instanced draw calls, 100k+ capable)
- Same interaction layer (pan/zoom/drag) shared between both
- Same data format (GraphNode[], GraphEdge[]) shared between both

## 5. Sim UI Component Pattern

Sim UI components are copy-paste, not npm-installable. They depend on `@spartan-ng/brain` for headless primitives and use `class-variance-authority` (cva) + `clsx` for variant styling. Import from `@spartan-ng/brain/button`, `@spartan-ng/brain/dialog`, etc.

The `@spartan-ng/helm/*` path is a LOCAL alias (set in tsconfig.json paths), not an npm package. Sim UI maps it to `./src/libs/ui/*/src/index.ts`.

## 6. Tauri Event Streaming for Graph Layout

Progressive graph layout uses Tauri events (not invoke responses) for streaming position updates:
```rust
app_handle.emit("graph-positions", BatchPayload { ... })?;
```
```typescript
import { listen } from '@tauri-apps/api/event';
const unlisten = await listen('graph-positions', (event) => { ... });
```
This keeps the invoke channel free for control commands and allows granular cancelation.

## 7. CSS Variable Theming with Tailwind v4

Tailwind v4 uses `@theme` for design tokens and CSS custom properties for runtime theming. The pattern is:
```css
:root {
  --primary: oklch(0.205 0 0);
  --background: oklch(1 0 0);
  --sidebar: oklch(0.985 0 0);
}
@theme inline {
  --color-primary: var(--primary);
  --color-background: var(--background);
  --color-sidebar: var(--sidebar);
}
```
This enables both light/dark mode (`:root.dark`) and runtime theme switching.
