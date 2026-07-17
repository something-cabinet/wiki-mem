---
title: Sim UI Component Integration
type: spec
tags: [spec, angular, ui, components]
---

## Overview

Replace hand-rolled Tailwind CSS components in the Angular web UI with [Sim UI](https://simui.dev) components — a free, open-source collection of copy-paste Angular components built on [Spartan UI](https://spartan.ng) and Tailwind CSS. This gives us production-ready, accessible components (Button, Input, Dialog, Select, Badge, Card, etc.) without maintaining custom component code.

## Locked Decisions

- D1: Use Sim UI's copy-paste approach — components are copied into the project codebase, not installed as a dependency
- D2: Spartan UI handles headless/accessible primitives (via Radix), Sim UI provides the visual layer
- D3: Tailwind CSS stays as the styling engine
- D4: Designer (human) reviews and selects which Sim UI components to use before implementation

## Requirements

### Functional Requirements

- FR-1: Replace existing plain Tailwind search input and Search button with Sim UI Input + Button components
- FR-2: Replace task board column headers (currently collapsible divs) with Sim UI Accordion or Collapsible
- FR-3: Replace "Create Page" and "New Memory" modals with Sim UI Dialog component
- FR-4: Replace memory layer/status filter dropdown with Sim UI Select component
- FR-5: Replace type filter pills (All, Pages, Tasks, Memory) with Sim UI Badge or Toggle Group
- FR-6: Replace page/memory result cards with Sim UI Card component
- FR-7: Add Sim UI Notification component for success/error feedback on actions
- FR-8: Replace navigation sidebar links with Sim UI Nav or appropriate link components
- FR-9: Install Spartan UI primitives (@spartan-ng/ui-*-helm, @radix-ng/primitives)
- FR-10: Copy selected Sim UI component source into apps/wm-web/src/app/components/

### Non-Functional Requirements

- NFR-1: Existing layout and responsive behavior must be preserved
- NFR-2: No regressions in existing functionality (search, navigation, CRUD)
- NFR-3: Component source is owned in-repo (copy-paste model) — no locked external API
- NFR-4: Accessibility must be maintained or improved (Spartan UI uses Radix, which is WAI-ARIA compliant)

## Acceptance Criteria

- [ ] AC-1: Spartan UI dependencies installed and configured
- [ ] AC-2: Search input + button replaced with Sim UI variants, search still works
- [ ] AC-3: Create Page and New Memory modals use Sim UI Dialog
- [ ] AC-4: Memory filter dropdown uses Sim UI Select
- [ ] AC-5: Type filter pills use Sim UI Badge or Toggle Group
- [ ] AC-6: Result cards use Sim UI Card
- [ ] AC-7: Task board columns use Sim UI Accordion/Collapsible
- [ ] AC-8: Success/error notifications appear on create/delete actions
- [ ] AC-9: Navigation sidebar preserved
- [ ] AC-10: All existing E2E journeys still pass
- [ ] AC-11: Designer has reviewed and approved component choices

## Scenarios

### Scenario 1: Search Page Refresh
**Given** a user on the Search page
**When** the page loads
**Then** the search input uses Sim UI Input styling
**And** the Search button uses Sim UI Button styling
**And** type filter pills use Sim UI Badge or Toggle Group
**And** search results display in Sim UI Cards

### Scenario 2: Create Page Modal
**Given** a user on the Pages list
**When** they click "Create Page"
**Then** a Sim UI Dialog opens with form fields using Sim UI Input and Select
**And** pressing Cancel closes the dialog
**And** pressing Create submits and shows a Notification

### Scenario 3: Task Board
**Given** a user on the Tasks page
**When** the board loads
**Then** each status column header is a Sim UI Accordion trigger
**And** clicking a header collapses/expands its task list

### Scenario 4: Memory Filters
**Given** a user on the Memory page
**When** they interact with the layer filter
**Then** it uses Sim UI Select instead of a plain `<select>`

## Technical Notes

### Spartan UI Installation
Spartan UI provides individual component packages:
- `@spartan-ng/ui-core` — core utilities
- `@spartan-ng/ui-button-helm` — button directive
- `@spartan-ng/ui-input-helm` — input directive  
- `@spartan-ng/ui-dialog-helm` — dialog components
- `@spartan-ng/ui-select-helm` — select components
- `@spartan-ng/ui-badge-helm` — badge directive
- `@spartan-ng/ui-card-helm` — card components
- `@spartan-ng/ui-accordion-helm` — accordion components

These depend on `@radix-ng/primitives` for headless behavior.

### Sim UI Component Source
Sim UI components are copied into `apps/wm-web/src/app/components/`. Each component gets its own folder with `.ts` and optionally `.html` files. The current views import them directly.

### Current Views to Modify

| View | File | Components to Replace |
|------|------|----------------------|
| Search | `search-view.component.ts` | Input, Button, Badge/Toggle, Card |
| Tasks | `tasks-view.component.ts` | Accordion, Badge |
| Pages | `pages-view.component.ts` | Dialog, Button, Input, Select, Card |
| Memory | `memory-view.component.ts` | Dialog, Select, Input, Button, Card |
| Graph | `graph-view.component.ts` | Input, Button, Card |
| Settings | `settings-view.component.ts` | Card, Button |
| Layout | `layout.component.ts` | Nav/sidebar links |

### Implementation Order
1. Install Spartan UI dependencies + configure
2. Copy Sim UI Button + Input — replace in Search view first
3. Copy Sim UI Dialog — replace in Pages and Memory modals
4. Copy Sim UI Badge + Card — replace type filters and result cards
5. Copy Sim UI Select — replace memory filters
6. Copy Sim UI Accordion — replace task board columns
7. Copy Sim UI Notification — wire into create/delete actions
8. Designer review pass

## Open Questions

- [ ] Should we use Sim UI's Dark Mode theme variant? (Current app has no dark mode)
- [ ] Should the sidebar navigation use Sim UI Nav components or stay as-is?
- [ ] What notification style (toast vs inline) for success/error feedback?
