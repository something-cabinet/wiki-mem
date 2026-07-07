---
id: snp52n
title: Fix skill parser for subdirectory format + name field
status: done
priority: high
labels:
  - from-spec
  - go-mode
createdAt: '2026-07-07T03:39:54.931Z'
updatedAt: '2026-07-07T03:46:18.923Z'
timeSpent: 0
spec: specs/wm-sdd-skills
fulfills:
  - AC-1
  - AC-2
---
# Fix skill parser for subdirectory format + name field

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Fix `parse_skill_file()` in skill.rs: (1) Detect subdirectory format (`wm-*/SKILL.md`) and use parent directory name as skill name, not `file_stem()`. (2) Read `name:` frontmatter field as primary name source with fallback to parent dir name. (3) Add `pub fn load_skills_from_embed()` that reads from rust-embed into the SkillEngine. (4) Update `scan()` to also load embedded skills.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

