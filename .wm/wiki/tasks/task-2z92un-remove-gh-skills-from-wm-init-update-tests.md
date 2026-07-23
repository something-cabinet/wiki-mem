---
title: Remove gh-* skills from wm init + update tests
type: task
status: done
tags: [from-spec, go-mode]
priority: high
knowns_id: 2z92un
spec: specs/wm-sdd-skills
fulfills: [AC-9, AC-10, AC-11, AC-12, AC-13, AC-14, AC-15]
relates_to:
  - {type: implements, target: wiki:specs:wm-sdd-skills}
---

# Remove gh-* skills from wm init + update tests

> **Spec:** `specs/wm-sdd-skills`

> **Fulfills:** AC-9, AC-10, AC-11, AC-12, AC-13, AC-14, AC-15

> *Imported from Knowns task `2z92un`*

# Remove gh-* skills from wm init + update tests

## Description


(1) Remove `generate_default_skills()` call from wm init. (2) Remove `generate_default_skills()` function from skill.rs. (3) Remove all gh-* string references. (4) Update `test_generate_default_skills` to test embedded skill loading instead. (5) Add tests for subdirectory parsing, `name:` field, platform sync helpers. (6) Ensure `wm serve` still scans `.agents/skills/` for runtime loading.


## Acceptance Criteria
