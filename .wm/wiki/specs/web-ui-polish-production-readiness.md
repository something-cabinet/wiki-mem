---
title: Web UI Polish — Production Readiness
type: spec
status: reviewed
tags:
  - spec
  - web-ui
  - angular
  - axum
---

# Spec: Web UI Polish — Production Readiness

## Locked Decisions (from Socratic Exploration)

- **D-1:** Sim UI (Spartan UI + Tailwind) is the component library for the Angular UI
- **D-2:** Component testing uses Jest with custom shallow stubs
- **D-3:** Full settings panel exposed in the Web UI — all user-configurable values
- **D-4:** Wave 1 (error/empty states + responsive sidebar) is the first delivery wave
- **D-5:** API errors return structured `{ code, message, hint }` matching wm-core's ToolError format
- **D-6:** RESTful resource endpoints for CRUD operations; RPC-style for search/rebuild
- **D-7:** NgRx for state management (store, effects, selectors)

## Overview
The Web UI (Angular 19 + Axum) was scaffolded in Sprint 5 with 6 views and a REST API, but it's not production-ready. The designer review rated it **4.4/10**. This spec covers the work needed to reach a **usable, resilient, and polished** state.

## Current State

| View | Read | Search | Create | Edit | Delete | Error handling | Empty states |
|---|---|---|---|---|---|---|---|
| Search | ✅ | ✅ | — | — | — | ✅ | ✅ |
| Graph | ✅ | ✅ | — | — | — | ❌ | ❌ |
| Tasks | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Pages | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Memory | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Settings | ⚠️ | — | — | — | — | ❌ | ❌ |

## Requirements

### FR-1: Mutations — Create, Edit, Delete in All Views

Users can't do anything with the Web UI beyond reading. Every view needs mutations:

- **Tasks**: Move between status columns (drag-and-drop kanban), edit title/priority/assignee, create new task, delete
- **Pages**: Create new page with type selector, edit content, delete
- **Memory**: Add/edit/delete entries across all 3 layers (project/global/session)
- **Settings**: Actual configurable settings (search mode, AI model, recency model)

### FR-2: Angular Modernization — Signals + OnPush

Zero signals used. All components use plain class properties triggering full Zone.js change detection.

- Refactor all 6 view components + layout to use `signal()`/`computed()` instead of class properties
- Set `changeDetection: ChangeDetectionStrategy.OnPush` on all components
- Use `toSignal` for HTTP observables where appropriate

### FR-3: Responsive Layout

The sidebar is a fixed `w-56` with no collapse. On mobile it's unusable.

- Implement collapsible sidebar with hamburger toggle
- Slide-out overlay drawer on mobile (< 768px)
- Persistent sidebar on desktop (≥ 768px)
- Sidebar state stored in a service (so it persists across route changes)

### FR-4: Error Handling + Empty States

Tasks, Memory, and Settings views have zero error handling. The app appears frozen when the API is down.

- Every view must handle: loading state (skeleton), error state (alert + retry button), empty state (illustration + CTA)
- Consistent error pattern: `ErrorComponent` or inline error with `role="alert"`
- Consistent loading pattern: `LoadingSkeletonComponent` with shimmer animation
- Consistent empty state: `EmptyStateComponent` with icon + message + suggested action

### FR-5: Visual Polish

- **Sim UI integration**: Install and configure Sim UI components (buttons, cards, badges, dialogs, inputs, selects)
- **Icon library**: Replace emoji sidebar icons (🔍🔗📋📄🧠⚙️) with proper SVG icons (Lucide or Sim UI icons)
- **Dark mode**: Implement with Tailwind `dark:` variants + a theme toggle in the sidebar
- **Loading skeletons**: Replace plain text "Loading..." with skeleton/shimmer components
- **Animations**: Page transitions, list stagger, hover micro-interactions

### FR-6: Accessibility

Only Search view has ARIA attributes. All other views need:

- `aria-label` on interactive elements (inputs, buttons, links)
- `role="status"` on loading states
- `role="alert"` on error states
- Focus management after search results load, page navigation, etc.
- Color-independent priority indicators (add text labels alongside color codes)

### FR-7: Backend Hardening

- **Audit trail**: Web mutations (create/update/delete) must write audit events to `.wm/log.jsonl`
- **CORS**: Restrict to localhost origins in production
- **Cache headers**: Set `Cache-Control: max-age=31536000, immutable` on hashed static assets; `no-cache` on `index.html`
- **Request logging**: Add `TraceLayer` middleware for API request logging
- **Error responses**: Standardize error response format across all endpoints

## Acceptance Criteria

- [ ] AC-1: Users can create, edit, and delete tasks via the Web UI
- [ ] AC-2: Users can create, edit, and delete wiki pages via the Web UI  
- [ ] AC-3: Users can create, edit, and delete memory entries via the Web UI
- [ ] AC-4: All 6 components use signals/OnPush change detection
- [ ] AC-5: Sidebar collapses on mobile, persistent on desktop
- [ ] AC-6: Every view handles loading/error/empty states with proper components
- [ ] AC-7: Sim UI components are installed and used (buttons, cards, badges, dialogs)
- [ ] AC-8: Emoji icons replaced with SVG icon library
- [ ] AC-9: Dark mode works across all views
- [ ] AC-10: ARIA attributes present on all interactive elements
- [ ] AC-11: Web mutations write audit events
- [ ] AC-12: CORS restricted to localhost in production
- [ ] AC-13: Static assets have correct cache headers
- [ ] AC-14: `cargo clippy -- -D warnings` passes
- [ ] AC-15: All existing tests still pass

## Non-Goals

- Graph visualization (D3.js/vis.js) — the current text-list graph view is acceptable for MVP
- Real-time collaboration (the tool is single-user local)
- Auth/password protection (local tool, by design)
- Push notifications

## Technical Notes

- Sim UI (https://simui.dev/) is an Angular component library built on Spartan UI + Tailwind CSS. Install via `npm install @sim-io/ui`.
- For icons, `lucide-angular` is the simplest option: `npm install lucide-angular`.
- Dark mode: add `dark:` variants to Tailwind config, store preference in `localStorage`, apply via class on `<html>`.
- Audit events: reuse the existing `emit_audit` / `EngineState::emit_audit` infrastructure from `wm-core`. The web server currently bypasses it (constructs `EngineState` directly without audit consumer).

## Delivery Sequence

| Wave | Items | Effort |
|------|-------|--------|
| **Wave 1** | FR-4 (error/empty states) + FR-3 (responsive layout) | 3 days |
| **Wave 2** | FR-2 (signals/OnPush) + FR-6 (accessibility) | 3 days |
| **Wave 3** | FR-1 (mutations — tasks + pages + memory) | 5 days |
| **Wave 4** | FR-5 (Sim UI, icons, dark mode, animations) | 4 days |
| **Wave 5** | FR-7 (audit, CORS, caching, logging) | 2 days |

## Open Questions

- [ ] OQ-1: Graph visualization — text-list acceptable for MVP (per non-goals). Revisit later.
- ~~OQ-2: Settings scope — resolved (D-3: full settings panel)~~
