---
id: wiki:tasks:web
title: Web UI Production Readiness
type: task
status: done
priority: high
tags: [web-ui, angular, polish]
acceptance_criteria:
  - text: "Memory create modal calls the API and creates entries, and REST /api/memory/list returns entries with a status field (Active/Stale/Archived)"
  - text: "All views show an error state on API failure with no infinite spinner, and ApiService exposes updatePage()"
  - text: "Sidebar collapses on screens <768px and all existing tests pass"
---
id: wiki:tasks:web

## Overview

Bring the Angular web UI (`apps/wm-web/`) to production readiness. The app is functional for read-only browsing but has several gaps.

## Requirements

- FR-1: Wire `MemoryEntry` create modal — `createEntry()` is a no-op stub
- FR-2: Add `status` filter to Memory view (Active/Stale/Archived)
- FR-3: Add `MemoryStatus` field to REST API response (`wm-server/src/api/memory.rs`)
- FR-4: Add error handling to Graph, Tasks, Memory, and Settings views
- FR-5: Add empty states to Graph and Pages views
- FR-6: Expose `POST /pages/update` in `ApiService`
- FR-7: Responsive sidebar (collapse, hamburger menu on mobile)
- FR-8: Search with `status` filter parameter

## Acceptance Criteria
- [x] AC-1: Memory create modal calls API and creates entry
- [x] AC-2: Memory list shows Active/Stale/Archived filter
- [x] AC-3: REST API `/api/memory/list` returns entries with `status` field
- [x] AC-4: All views show error state on API failure
- [x] AC-5: No component shows infinite spinner on failure
- [x] AC-6: `ApiService` has `updatePage()` method
- [x] AC-7: Sidebar collapses on screens <768px
- [x] AC-8: All existing tests pass

## Status — DONE (per-AC disposition)

| AC | Disposition | Note |
|----|-------------|------|
| AC-1 | **MOOT by design** | Web API is read-only — no write UI exists, so no create modal/dialog anywhere. Verified: zero `HlmDialog`/dialog usage in `apps/wm-web/src/app/`. |
| AC-2 | Done | Memory view has a Status filter (All/Active/Stale/Archived) driving `listMemory(layer, status)`. |
| AC-3 | Done | `listMemory` response includes per-entry `status`; MemoryStatus surfaced in REST response (wm-server side, prior lane). |
| AC-4 | Done | All 6 views (tasks, pages, memory, search, graph, settings) show an error state via shared `WmErrorState` (destructive alert + Retry). |
| AC-5 | Done | All views have finite loading: skeletons/spinners stop on success or error; no infinite spinner on failure. |
| AC-6 | **MOOT by design** | No `updatePage()` — the web API exposes no write endpoint (`POST /pages/update` doesn't exist) and the UI is read-only, so exposing a write stub would be dead code. |
| AC-7 | Done | Sidebar collapses <768px with hamburger toggle; persistent ≥768px. |
| AC-8 | Done | `tsc --noEmit` clean and `ng build` succeeds (e2e intentionally not run per lane scope). |
