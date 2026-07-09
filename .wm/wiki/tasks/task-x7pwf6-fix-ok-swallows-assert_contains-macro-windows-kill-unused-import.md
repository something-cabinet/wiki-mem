---
title: Fix .ok() swallows, assert_contains! macro, Windows kill, unused import
type: task
status: done
tags: [review-fix, test-infra]
priority: medium
knowns_id: x7pwf6
---

# Fix .ok() swallows, assert_contains! macro, Windows kill, unused import

> *Imported from Knowns task `x7pwf6`*

# Fix .ok() swallows, assert_contains! macro, Windows kill, unused import

## Description


P2 items from rust-reviewer:
- 9 .ok() calls in setup_test_project() → .expect() with messages
- assert_contains! macro double-evaluated $haystack → bind to locals
- run_cli_with_timeout used Windows-only taskkill → cross-platform kill_process() helper
- Removed unused Read from module-level import

All done.


## Acceptance Criteria
