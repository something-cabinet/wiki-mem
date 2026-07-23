---
title: "Fix Tauri Task Board — Status Hardcoded to Draft"
type: spec
status: approved
tags: [spec, tauri, web-ui, task-board, bug]
relates_to:
  - {type: implements, target: "wiki:tasks:fix-tauri-task-board-status-hardcoded-draft"}
---

## Overview

The Tauri backend `commands.rs` always reports every task as `Draft` status in the web UI, regardless of each task's actual frontmatter `status` field. This affects both the task board view (`/tasks`) and individual page views that display status.

## Locked Decisions

- D1: Fix both `task_board` and `get_page` code paths — the `SimplePageMeta` deserialization fallback exists in two locations
- D2: Parse status from frontmatter string using `PageStatus` deserialization, fall back to `PageStatus::Draft` only if the status value itself is invalid
- D3: Add `priority` field to `SimplePageMeta` and parse it the same way — fall back to `None` if missing or invalid (matching `WikiPageMeta` behavior)

## Root Cause

In `apps/wm-web/src-tauri/src/commands.rs`, the `read_all_pages` helper and `get_page` handler both use a two-tier deserialization:

1. Try `serde_yaml::from_str::<WikiPageMeta>(fm)` — full metadata struct with all per-type fields
2. If that fails, fall back to `serde_yaml::from_str::<SimplePageMeta>(fm)` — minimal field set

The `SimplePageMeta` struct correctly parses `status: String` from frontmatter, but the code that constructs the `WikiPageMeta` output ignores both `simple.status` and `simple.priority`, hardcoding defaults:

```rust
// Line 146 — read_all_pages (affects task_board)
status: wm_core::engine::PageStatus::Draft,
priority: None,

// Line 210 — get_page
status: wm_core::engine::PageStatus::Draft,
priority: None,
```

Additionally, `SimplePageMeta` doesn't declare a `priority` field at all, so any `priority:` value in frontmatter is silently dropped before the fallback even runs.

Most task wiki pages do not have the full `WikiPageMeta` optional per-type fields (e.g., `task_data`, `spec_data`), so they always hit the `SimplePageMeta` fallback, resulting in all tasks appearing as "Draft" with no priority in the web UI regardless of their actual frontmatter.

## Requirements

### Functional Requirements

- FR-1: Task board view shows each task with its correct status from frontmatter
- FR-2: Page detail view shows correct status for each page
- FR-3: Invalid or missing status in frontmatter falls back to `Draft` (graceful degradation)
- FR-4: Task board view shows each task's correct priority (high, medium, low, urgent) from frontmatter

### Non-Functional Requirements

- NFR-1: Zero behavioral change for any code path not hitting the `SimplePageMeta` fallback
- NFR-2: Build passes with zero errors (`cargo build`)

## Acceptance Criteria

- [ ] AC-1: `wm cli mcp` task board shows the same status as the web UI for any given task
- [ ] AC-2: A task with `status: done` in its `.md` file appears in the "done" column of the web UI task board
- [ ] AC-3: A task with `status: cancelled` appears in the "cancelled" column
- [ ] AC-4: A page with invalid status (e.g., `status: foobar`) defaults to `Draft` without error
- [ ] AC-5: A page with no `status` field in frontmatter defaults to `Draft`
- [ ] AC-6: `get_page` returns correct `status` in its JSON response
- [ ] AC-7: `get_page` returns correct `priority` in its JSON response
- [ ] AC-8: A task with `priority: high` in frontmatter shows the high-priority indicator (red left border) in the task board
- [ ] AC-9: A task with no `priority` field defaults to no priority indicator in the web UI
- [ ] AC-10: `cargo build` passes with zero warnings

## Scenarios

### Scenario 1: Task Board Shows Mixed Statuses
**Given** the wiki has tasks with `status: todo`, `status: done`, `status: cancelled`
**When** the user navigates to `/tasks` in the web UI
**Then** tasks appear in their respective status columns (todo, done, cancelled)
**And** no tasks appear under "draft" unless they actually have `status: draft`

### Scenario 2: Invalid Status Graceful Fallback
**Given** a wiki task page has `status: unknown_value` in its frontmatter
**When** the task board loads
**Then** the task appears in the "draft" column without error

### Scenario 3: `get_page` Shows Correct Status
**Given** a task page with `status: in-progress`
**When** the user opens the page detail view
**Then** the status is displayed as "in-progress"

### Scenario 4: Priority Indicator Displayed
**Given** a task with `priority: high` in its frontmatter
**When** the task board loads
**Then** the task card shows a red left border (high-priority indicator)

## Technical Notes

### Status fix

In two locations in `commands.rs` (lines 146 and 210), replace:

```rust
status: wm_core::engine::PageStatus::Draft,
```

With:

```rust
status: serde_yaml::from_str(&simple.status)
    .unwrap_or(wm_core::engine::PageStatus::Draft),
```

### Priority fix

Add `priority: Option<String>` to `SimplePageMeta`:

```rust
#[derive(Deserialize)]
struct SimplePageMeta {
    // ...existing fields...
    #[serde(default)]
    priority: Option<String>,
}
```

Then in both fallback locations, replace:

```rust
priority: None,
```

With:

```rust
priority: simple.priority
    .and_then(|p| serde_yaml::from_str::<wm_core::engine::Priority>(&p).ok()),
```

This returns `None` if the `priority` field is absent from frontmatter or contains an invalid value — matching the existing default behavior but respecting valid values when present.

No new imports needed — `serde_yaml` is already a dependency. `PageStatus` and `Priority` both derive `Deserialize` with `#[serde(rename_all = "kebab-case")]`.

## Open Questions

None — root cause, fix, and scope are fully understood.
