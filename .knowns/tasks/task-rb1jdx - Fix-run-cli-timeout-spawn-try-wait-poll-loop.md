---
id: rb1jdx
title: Fix run_cli() timeout — spawn + try_wait poll loop
status: done
priority: high
labels:
  - review-fix
  - test-infra
createdAt: '2026-07-07T08:50:57.488Z'
updatedAt: '2026-07-07T08:50:57.488Z'
timeSpent: 0
---
# Fix run_cli() timeout — spawn + try_wait poll loop

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
P0 from rust-reviewer. run_cli() claimed 60s timeout but used Command::output() which blocks indefinitely. Replaced with spawn + try_wait() poll loop with 60s deadline and cross-platform kill_process().
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

