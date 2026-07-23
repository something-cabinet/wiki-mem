---
title: TUI: search scrolling, Pagination, tab cycle unicode
type: task
status: done
tags: [review, tui, ratatui]
priority: medium
knowns_id: 75k8oh
spec: specs/tui-polish-search-scrolling-pagination-tab-cycle-unicode
relates_to:
  - {type: implements, target: wiki:specs:tui-polish-search-scrolling-pagination-tab-cycle-unicode}
---

# TUI: search scrolling, Pagination, tab cycle unicode

> **Spec:** `specs/tui-polish-search-scrolling-pagination-tab-cycle-unicode`

> *Imported from Knowns task `75k8oh`*

# TUI: search scrolling, Pagination, tab cycle unicode

## Description


Fix designer-review TUI issues:

1. **Search results overflow** (tui.rs) — Replace Paragraph with List + scroll state for search results (matching dashboard's >50 node pattern).

2. **Search preview scrolling** (tui.rs) — Add preview_scroll to App. Handle Up/Down/PgUp/PgDown in preview mode.

3. **PageUp/PageDown in Dashboard** (tui.rs) — Scroll 10 items at a time.

4. **Tasks tab in Tab/Shift+Tab cycle** (tui.rs) — Include Tasks in the Help→Search→Graph→Dashboard→Tasks→Help cycle.

5. **Unicode box-drawing fallback** (tui.rs) — Add ASCII fallback for terminals without Unicode support.


## Acceptance Criteria



## Implementation Notes


TUI polish implemented:
- Search results: List + Scrollbar instead of Paragraph, matching dashboard pattern
- Preview scrolling: preview_scroll field, Up/Down/PgUp/PgDown in preview mode
- Dashboard: PgUp/PgDown scroll 10 items
- Tab cycle: Help→Search→Graph→Dashboard→Tasks→Help
- Unicode fallback: ASCII_BORDER, block_bordered() helper, inline ASCII for graph/task/help symbols
wm-cli builds.
