---
id: 2z92un
title: 'Remove gh-* skills from wm init + update tests'
status: done
priority: high
labels:
  - from-spec
  - go-mode
createdAt: '2026-07-07T03:39:58.748Z'
updatedAt: '2026-07-07T03:46:19.712Z'
timeSpent: 0
spec: specs/wm-sdd-skills
fulfills:
  - AC-9
  - AC-10
  - AC-11
  - AC-12
  - AC-13
  - AC-14
  - AC-15
---
# Remove gh-* skills from wm init + update tests

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
(1) Remove `generate_default_skills()` call from wm init. (2) Remove `generate_default_skills()` function from skill.rs. (3) Remove all gh-* string references. (4) Update `test_generate_default_skills` to test embedded skill loading instead. (5) Add tests for subdirectory parsing, `name:` field, platform sync helpers. (6) Ensure `wm serve` still scans `.agents/skills/` for runtime loading.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

