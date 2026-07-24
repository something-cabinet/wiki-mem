---
title: TUI Polish — Search Scrolling, Pagination, Tab Cycle, Unicode
type: spec
tags:
  - spec
  - approved
  - tui
  - ratatui
---
id: wiki:specs:tui-polish-search-scrolling-pagination-tab-cycle-unicode

## Overview

Fix 5 TUI polish items identified during designer review for the Ratatui terminal interface. These focus on scroll ergonomics, navigation completeness, and terminal compatibility.

## Locked Decisions

- D1: Search results overflow → replace Paragraph with List + scroll state (matching dashboard pattern)
- D2: Search preview → add preview_scroll to App state
- D3: PageUp/PageDown → scroll 10 items at a time (matching terminal conventions)
- D4: Tasks tab → include in Help→Search→Graph→Dashboard→Tasks→Help cycle
- D5: Unicode box-drawing → fall back to ASCII when terminal doesn't support Unicode

## Requirements

### Functional Requirements

- FR-1: Search results must scroll when they exceed the viewport
- FR-2: Preview mode in search must support vertical scrolling
- FR-3: Dashboard must scroll 10 items per PageUp/PageDown
- FR-4: Tab/Shift+Tab must cycle through all 5 tabs, including Tasks
- FR-5: Terminals without Unicode support must use ASCII box-drawing fallback

### Non-Functional Requirements

- NFR-1: Existing TUI functionality must not regress
- NFR-2: `cargo build` and `cargo test` pass without new warnings

## Acceptance Criteria

- [ ] AC-1: Search results (tui.rs) use a List widget with scroll state instead of Paragraph
- [ ] AC-2: App state has `preview_scroll` field for search preview scrolling
- [ ] AC-3: Up/Down/PgUp/PgDown work in search preview mode
- [ ] AC-4: PageUp/PageDown in Dashboard scroll 10 items at a time
- [ ] AC-5: Help→Search→Graph→Dashboard→Tasks→Help tab cycle works with Tab/Shift+Tab
- [ ] AC-6: Unicode box-drawing has ASCII fallback for terminals without Unicode
- [ ] AC-7: All existing TUI tests pass

## Scenarios

### Scenario 1: Long search results
**Given** a search returning 100+ results
**When** results are displayed in the TUI
**Then** results are in a scrollable List; Up/Down arrows scroll; PageUp/PageDown scroll 10 at a time

### Scenario 2: No Unicode terminal
**Given** a terminal without Unicode box-drawing support
**When** the TUI renders borders and dividers
**Then** ASCII characters (+, -, |) are used instead of Unicode (┌, ─, │)

### Scenario 3: Tab navigation
**Given** the TUI is in any tab
**When** user presses Tab repeatedly
**Then** focus cycles through Help → Search → Graph → Dashboard → Tasks → Help

## Technical Notes

- All changes in wm-cli/src/tui.rs
- Dashboard already has scroll state pattern (from task 6lzncr) — search should follow same pattern
- Unicode detection can use `std::env::var("TERM")` or detect via terminal capabilities
