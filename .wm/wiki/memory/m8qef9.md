---
title: WM-Knowns tool surface gaps
type: memory
tags: [knowns, wm, gap-analysis, tool-surface, reference]
created_at: "2026-07-09T18:26:25.233Z"
updated_at: "2026-07-09T18:26:25.233Z"
---

# WM-Knowns Tool Surface Gap Analysis

Generated: 2026-07-10

## Overview

Knowns exposes 8 unified tools (code, docs, memory, project, search, tasks, templates, time, validate) each with an `action` parameter for sub-operations. WM exposes 17+ domain-module "wm_*" tools with dot-notation (e.g., `wm_task.create`) — same concepts, different naming.

Both systems have equivalent coverage for all major operations. Gaps are at the **parameter/feature level**.

---

## Tool-by-Tool Comparison

### 1. code → wm_code
**Status: PARITY** ✅ (WM uses regex-based code intelligence; knowns uses AST-based)

| Feature | Knowns | WM | Notes |
|---------|--------|----|-------|
| Code search (text) | `code` action:search, query param | `wm_code.search` pattern param | Knowns = AST-aware, WM = regex |
| Symbol lookup | `code` action:symbols | `wm_code.symbols` | Knowns = AST, WM = regex for Rust |
| Dependency graph | `code` action:deps | `wm_code.deps` | Same concept |
| Graph traversal | `code` action:graph | (none in wm_code) | Knowns-only: graph edge expansion from code matches |

### 2. docs → wm_doc
**Status: FUNCTIONAL GAPS ⚠️**

| Feature | Knowns | WM | Priority |
|---------|--------|----|----------|
| CRUD (create/get/update/delete/list) | ✅ | ✅ | - |
| Doc history | ✅ action:history | ❌ | Low |
| Section-level get/update | ✅ section, toc, line params | ❌ | Medium |
| `appendContent` for update | ✅ | ❌ (must read+write full) | **High** (referenced in 4 skills) |
| `smart` mode (auto TOC on large docs) | ✅ | ❌ | Low |
| `info` (stats only) | ✅ | ❌ | Low |
| `clear` (clear individual fields) | ✅ | ❌ | Low |
| `newPath` (rename) | ✅ | ❌ | Low |
| `folder` (create in folder) | ✅ | ❌ | Low |
| `dryRun` for delete | ✅ | ❌ | Low |

### 3. memory → wm_memory
**Status: NEAR PARITY ⚠️**

| Feature | Knowns | WM | Priority |
|---------|--------|----|----------|
| CRUD | ✅ | ✅ | - |
| promote (project→global) | ✅ | ✅ | - |
| demote (global→project) | ✅ | ❌ | Low |
| `category` field | ✅ | ✅ | - |
| `tag` filter in list | ✅ | ✅ | - |
| `dryRun` for delete | ✅ | ❌ | Low |
| `layer` for add (project/global/session) | ✅ | ✅ | - |

### 4. project → wm_project
**Status: NEAR PARITY ⚠️**

| Feature | Knowns | WM | Priority |
|---------|--------|----|----------|
| detect | ✅ | ✅ | - |
| current | ✅ | ❌ (no wm_project.current) | Low |
| set | ✅ | ✅ | - |
| status | ✅ | ✅ | - |
| additionalPaths for detect | ✅ | ❌ | Low |

### 5. search → wm_search
**Status: NEAR PARITY ⚠️**

| Feature | Knowns | WM | Priority |
|---------|--------|----|----------|
| query → q param naming | `query` | `q` | - |
| search (keyword/semantic/hybrid) | ✅ | ✅ | - |
| retrieve (context assembly) | ✅ | ✅ | - |
| resolve (query→ID) | ✅ | ✅ | - |
| type filter (all/task/doc/memory) | ✅ | ✅ | - |
| sourceTypes (retrieve filter) | ✅ itemized array | ❌ (only type string) | Low |
| expandReferences (retrieve) | ✅ | ❌ | Medium |
| assignee/label/priority/status/tag filters for search | ✅ | ❌ | Low |
| direction/depth/entityTypes/relationTypes for resolve | ✅ | ❌ | Low |

### 6. tasks → wm_task
**Status: NEAR PARITY ⚠️**

| Feature | Knowns | WM | Priority |
|---------|--------|----|----------|
| CRUD (create/get/update/delete/list) | ✅ | ✅ | - |
| check_ac / uncheck_ac | ✅ | ✅ | - |
| board | ✅ | ✅ | - |
| history | ✅ action:history | ❌ | Low |
| appendNotes | ✅ | ❌ (no wm_task equivalent) | Low |
| parent (subtask relationships) | ✅ | ❌ | Low |
| fulfills / spec | ✅ | ❌ | Low |
| plan field | ✅ | ❌ | Low |
| order / label filter | ✅ | ❌ | Low |
| dryRun for delete | ✅ | ❌ | Low |
| `taskId` → `id` param naming | `taskId` | `id` | - |

### 7. templates → wm_template
**Status: PARITY** ✅

| Feature | Knowns | WM | Notes |
|---------|--------|----|-------|
| list/get/run | ✅ | ✅ | - |
| create | ✅ | ✅ | - |
| `doc` reference field | ✅ | ❌ | Low |
| `dryRun` as param | ✅ | ❌ (dryRun is top-level) | - |

### 8. time → wm_time
**Status: NEAR PARITY ⚠️**

| Feature | Knowns | WM | Priority |
|---------|--------|----|----------|
| start/stop/add/report | ✅ | ✅ | - |
| groupBy for report | ✅ | ❌ | Low |
| `taskId` → `id` param naming on start/stop | `taskId` | `id` | - |

### 9. validate → wm_validate
**Status: GAPS ⚠️**

| Feature | Knowns | WM | Priority |
|---------|--------|----|----------|
| `scope` (all/tasks/docs/templates/sdd) | ✅ | ✅ | - |
| `entity` (validate specific task/doc) | ✅ | ❌ | Medium |
| `fix` (auto-fix) | ✅ | ❌ | Medium |
| `strict` (warnings→errors) | ✅ | ❌ | Low |

---

## Parameter Name Differences

| Knowns param | WM param | Tools affected |
|-------------|----------|----------------|
| `query` | `q` | search, code |
| `taskId` | `id` | tasks, time |
| `tag` (filter) | same | docs, memory |
| `limit` | same | most tools |
| `category` | same | memory |
| `tags` | same | docs, memory, tasks |
| `scope` | same | validate |

---

## Priority Summary

### High (referenced in skills, needs fix)
1. **`appendContent` on wm_doc.update** — 4 skills use it, no WM equivalent
2. **`wm_template.create` missing `content`** required param — 2 skills reference
3. **`entity` param on validate** — needed for targeted validation

### Medium (improves workflow)
4. **Section-level doc access** (section, toc, line)
5. **expandReferences for retrieve**
6. **validate with entity/fix**

### Low (nice-to-have)
7. Doc history, smart mode, info, clear, newPath
8. memory demote
9. task history, appendNotes, parent, fulfills, plan
10. time groupBy