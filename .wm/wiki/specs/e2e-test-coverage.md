---
title: E2E Test Coverage
type: spec
status: draft
tags: [testing, e2e, coverage]
---

## Overview

Define end-to-end test cases for all major features implemented across the recent rework. Tests run the full CLI/MCP pipeline — create data via tools, verify via queries, validate persistence across restarts.

## Existing Coverage

Current E2E tests (3 tests in `tests/e2e_test.rs`):
- `test_full_workflow` — create all 7 page types, link, search, retrieve, graph, time tracking, lint, validate, rebuild
- `test_workflow_full_session` — init, create pages, link, search, graph, rebuild, validate, lint
- `test_state_machine_transitions` — task status transitions via board

## New Test Cases

### E2E-1: Memory as wiki pages

**Coverage**: Memory→Page migration, MemoryData fields, session memory

**Steps**:
1. Create a memory page: `wm page create memory/test-mem "Test Memory" --content "Test content" --page-type memory`
2. Verify it appears in `wm page list --json` with type `memory`
3. Search for "Test Memory" — verify it appears in search results
4. Create a second memory with `relates_to` linking to the first
5. Verify graph neighbors shows the edge
6. Add a memory via session layer (in-memory DashMap) — verify it's listed
7. Restart and verify session memory is gone but project memory persists

### E2E-2: Vector storage with turso

**Coverage**: VectorDb, hybrid search, index rebuild

**Steps**:
1. Create pages with rich content
2. Run `wm index rebuild`
3. Verify `.wm/state/vectors.db` exists
4. Search with hybrid mode — verify results are returned
5. Create a new page, run incremental index, verify it appears in search
6. Delete a page, reindex, verify it no longer appears

### E2E-3: Version history

**Coverage**: VersionStore, version list/get/rollback, FSRS compaction creation

**Steps**:
1. Create a task page
2. Update the task title via `wm page update`
3. Call `wm version list wiki:tasks:test-version` — verify version v1 exists
4. Call `wm version get wiki:tasks:test-version v1` — verify the diff shows title change
5. Update the task status, verify v2 is created
6. Call `wm version rollback wiki:tasks:test-version v1` — verify title reverts
7. Verify the page now has the original title

### E2E-4: Action-enum MCP tools

**Coverage**: Merged tool surface, action dispatch, invalid action handling

**Steps**:
1. Call `wm_page` with `{"action": "list"}` — verify returns page list
2. Call `wm_page` with `{"action": "get", "id": "wiki:..."}` — verify returns page content
3. Call `wm_page` with `{"action": "fly"}` — verify returns error for invalid action
4. Call `wm_task` with `{"action": "board"}` — verify returns task board
5. Call `wm_memory` with `{"action": "list"}` — verify returns memory list
6. Verify the old dot-notation tools (`wm_page.list`, `wm_page.get`) return error (not found — they were renamed)

### E2E-5: Status validation per page type

**Coverage**: `PageType::allowed_statuses()`, tool-layer validation

**Steps**:
1. Try `wm task create` with `--status approved` — verify error (approved not allowed for tasks)
2. Try `wm task update` setting `--status approved` — verify error
3. Try `wm decision create` with `--status in-progress` — verify error
4. Try `wm page create concepts/test` with `--status todo` — verify error (todo not allowed for concepts)
5. Create a task with `--status todo`, verify success

### E2E-6: Config enrichment

**Coverage**: StatusColors, visible columns, LSP, git tracking, runtime memory settings

**Steps**:
1. Read config via `wm project status --json` — verify default status colors exist
2. Verify task board uses the configured status columns
3. Modify config to set `visible_columns` to only ["todo", "done"]
4. Verify `wm task board` only shows "todo" and "done" columns

### E2E-7: @wiki references

**Coverage**: Reference format `@wiki/{type}/{name}`, extract and resolve

**Steps**:
1. Create a page with content containing `See @wiki/tasks/test-ref for details`
2. Call `wm ref.extract` on the page content — verify it extracts `@wiki/tasks/test-ref`
3. Call `wm ref.resolve @wiki/tasks/test-ref` — verify it resolves to the task content
4. Create a reference to a non-existent page — verify error
5. Verify code blocks skip references (existing behavior)

### E2E-8: Template prompt system

**Coverage**: `_template.yaml`, directory templates, actions (add, addMany)

**Steps**:
1. Create a directory template at `.wm/templates/e2e-test/_template.yaml` with:
   - One text prompt
   - One `add` action
2. Run `wm template run e2e-test` with variables — verify file is created
3. Run `wm template list` — verify directory template appears
4. Run `wm template get e2e-test` — verify config is returned

### E2E-9: Page update with typed params

**Coverage**: `PageUpdateParams`, field-level updates, atomic writes

**Steps**:
1. Create a page with initial title, content, status, tags
2. Update only the title — verify other fields unchanged
3. Update only tags — verify title and content unchanged
4. Update status — verify status validation fires (tied to E2E-5)
5. Update all fields at once — verify all changed

### E2E-10: Concurrent session state

**Coverage**: Session memory, in-memory DashMap, eviction

**Steps**:
1. Create session memory entries
2. Verify they appear in `wm memory list --layer session`
3. Verify they do NOT persist after engine restart

## Implementation Notes

- Use `setup_test_project()` helper from existing tests (creates temp directory with `.wm/` structure)
- Add new test file `tests/e2e_v2_test.rs` to keep old tests intact
- Each test should be independent — clean project per test
- Use `run_cli()` for CLI tests, `MCPClient` for MCP protocol tests
- Priority: E2E-1 (memory→pages) and E2E-3 (versions) are highest impact

## Acceptance Criteria

- [ ] E2E-1: Memory pages persist, searchable, graphable
- [ ] E2E-2: vectors.db exists after rebuild, search returns results
- [ ] E2E-3: Versions created on update, rollback restores state
- [ ] E2E-4: Action enums dispatch correctly, invalid actions error
- [ ] E2E-5: Invalid status per page type rejected
- [ ] E2E-6: Config fields readable/writable
- [ ] E2E-7: @wiki references extract and resolve
- [ ] E2E-8: Directory templates work end-to-end
- [ ] E2E-9: Field-level updates precise
- [ ] E2E-10: Session memory ephemeral, project memory persists
