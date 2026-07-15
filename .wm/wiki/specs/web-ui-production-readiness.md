---
title: Web UI Production Readiness
type: spec
status: draft
tags: [web-ui, angular, polish, production]
---

## Overview

The Angular web UI (`apps/wm-web/`) is functional for read-only browsing but has gaps preventing production use: memory create is a no-op stub, error handling is missing in half the views, MemoryStatus filtering is absent, and the API service is missing endpoints.

## Current State

| View | Create | Edit | Delete | Error State | Empty State | Status filter |
|------|--------|------|--------|-------------|-------------|---------------|
| Search | — | — | — | ✅ Alert | ✅ "No results" | — |
| Graph | — | — | — | ❌ Missing | ❌ Missing | — |
| Tasks | ❌ | ❌ | ❌ | ❌ Missing | ✅ "No tasks" | — |
| Pages | ✅ Create | ❌ | ❌ | ✅ Alert | ❌ Missing | — |
| Memory | ❌ Stub | ❌ | ❌ | ❌ Missing | ✅ "No entries" | ❌ Missing |
| Settings | — | — | — | ❌ Infinite spinner | — | — |

## Gray Areas

### GA1: Memory view — should MemoryStatus filter be a dropdown or chips?
Options: dropdown select (like layer filter), or pill/chip buttons for Active/Stale/Archived?

### GA2: Error handling approach — global interceptor or per-view?
Options: add an HTTP interceptor that catches errors globally, or add per-view error states?

### GA3: Responsive sidebar — slide-over or collapsible?
Options: slide-over drawer on mobile (like GitHub), or inline collapse/expand toggle?

### GA4: Memory create modal — should it use the wm_memory MCP tool or the wm_page API?
Memory is now a page type. Should the UI call `wm_page` or `wm_memory`?

## Requirements

### FR-1: Memory create wired
Replace the stub `createEntry()` with a real API call. Memory is now a page type — the UI should POST to `/api/pages/create` with `page_type: memory`.

### FR-2: MemoryStatus filter
Add a filter dropdown to the Memory view for `active`, `stale`, `archived`. Wire to `/api/memory/list?status=active`.

### FR-3: MemoryStatus in REST response
Update `wm-server/src/api/memory.rs` to include the `status` field in memory entry responses.

### FR-4: Error handling in all views
Every view must display an error message when API calls fail (no infinite spinners).

### FR-5: Empty states
Graph neighbors view and Pages list must show an empty state when no data.

### FR-6: ApiService.updatePage()
Add `updatePage(id, fields)` method calling `POST /api/pages/update`.

### FR-7: Responsive sidebar
The current fixed-width sidebar must collapse on narrow screens. At minimum: hamburger button, slide-over drawer.

### NFR-1: All existing tests must pass
No breaking changes to existing functionality.

## Acceptance Criteria

- [ ] AC-1: Memory create modal creates a memory page via API
- [ ] AC-2: Memory list has Active/Stale/Archived filter
- [ ] AC-3: REST API `/api/memory/list` returns `status` per entry
- [ ] AC-4: Graph view shows error when neighbor fetch fails
- [ ] AC-5: Tasks view shows error when board fetch fails
- [ ] AC-6: Memory view shows error when list fails
- [ ] AC-7: Settings shows error when status fetch fails
- [ ] AC-8: Graph neighbors shows "No neighbors" when empty
- [ ] AC-9: Pages list shows "No pages" when empty
- [ ] AC-10: ApiService has `updatePage(id, fields)` method
- [ ] AC-11: Sidebar collapses to hamburger on screens <768px
- [ ] AC-12: All existing Angular tests pass
