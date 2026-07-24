---
id: wiki:tasks:web
title: Web UI Production Readiness
type: task
status: todo
priority: high
tags: [web-ui, angular, polish]
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
- [ ] AC-1: Memory create modal calls API and creates entry
- [ ] AC-2: Memory list shows Active/Stale/Archived filter
- [ ] AC-3: REST API `/api/memory/list` returns entries with `status` field
- [ ] AC-4: All views show error state on API failure
- [ ] AC-5: No component shows infinite spinner on failure
- [ ] AC-6: `ApiService` has `updatePage()` method
- [ ] AC-7: Sidebar collapses on screens <768px
- [ ] AC-8: All existing tests pass
