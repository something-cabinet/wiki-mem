---
id: 8qeo96
title: Dead Code Cleanup
status: done
priority: medium
labels:
  - cleanup
  - chore
createdAt: '2026-07-06T17:40:24.331Z'
updatedAt: '2026-07-07T08:09:59.787Z'
timeSpent: 0
---
# Dead Code Cleanup

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Remove/verify: (1) first_non_empty and contains_str in util.rs (unused), (2) _meta assignments in page.rs:51,60, (3) Stale #[allow(dead_code)] on parse_priority (parser.rs:194 — actually used) and create_engine (main.rs:371 — actually used), (4) Unused import std::io::Write (engine.rs:6), (5) Unused _query_tokens in search.rs:316.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

