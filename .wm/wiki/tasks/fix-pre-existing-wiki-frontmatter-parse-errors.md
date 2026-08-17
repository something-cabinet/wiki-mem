---
title: Fix pre-existing wiki frontmatter parse errors
type: task
id: wiki:tasks:fix-pre-existing-wiki-frontmatter-parse-errors
status: todo
priority: low
tags: [bug, wiki-health, frontmatter]
acceptance_criteria:
  - text: "wm lint check reports no frontmatter parse errors for linus-core-simplicity-rule and graph-index-staleness-write-handlers-need-disk-fallback"
  - text: "Both pages parse cleanly with correct typed fields"
---

Pre-existing wiki parse errors surfaced by the CLI task board during the wm-doc-fix wave: (1) .wm/wiki/specs/linus-core-simplicity-rule.md - general_goals[0] invalid type: string, expected struct GoalEntry; (2) .wm/wiki/memory/graph-index-staleness-write-handlers-need-disk-fallback.md - mapping values not allowed (unquoted value with colon). Both break frontmatter parsing; fix the source pages (via wm_page update, never manual edit).