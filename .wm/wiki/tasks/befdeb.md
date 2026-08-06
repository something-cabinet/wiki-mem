---
title: Standardize page headers across all views
id: befdeb
type: task
status: done
priority: medium
tags: [ux, consistency, layout]
acceptance_criteria:
  - text: "All views (Graph, Settings, Search, Tasks, Pages, Memory) use a consistent header pattern based on the Graph header reference (bg-card, border-b, proper padding)"
  - text: "No view retains a divergent header pattern (plain h1 or standalone heading + button row)"
---

Each view has a different header pattern:
- Graph: header bar with badges
- Settings: heading + button row
- Search/Tasks/Pages/Memory: plain h1
Standardize to a consistent header pattern across all views. Use the Graph header as the reference (bg-card, border-b, proper padding).
