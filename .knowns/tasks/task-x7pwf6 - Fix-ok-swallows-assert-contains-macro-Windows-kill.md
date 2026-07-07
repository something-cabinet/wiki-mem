---
id: x7pwf6
title: 'Fix .ok() swallows, assert_contains! macro, Windows kill, unused import'
status: done
priority: medium
labels:
  - review-fix
  - test-infra
createdAt: '2026-07-07T08:51:00.392Z'
updatedAt: '2026-07-07T08:51:00.392Z'
timeSpent: 0
---
# Fix .ok() swallows, assert_contains! macro, Windows kill, unused import

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
P2 items from rust-reviewer:
- 9 .ok() calls in setup_test_project() → .expect() with messages
- assert_contains! macro double-evaluated $haystack → bind to locals
- run_cli_with_timeout used Windows-only taskkill → cross-platform kill_process() helper
- Removed unused Read from module-level import

All done.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

