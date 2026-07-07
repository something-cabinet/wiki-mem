---
id: 29fizw
title: 'WM SDD Skills: replace gh-* with wm-* workflow skills'
status: done
priority: high
labels:
  - skills
  - workflow
  - sdd
  - knowns
createdAt: '2026-07-07T03:22:29.328Z'
updatedAt: '2026-07-07T06:19:51.832Z'
timeSpent: 0
---
# WM SDD Skills: replace gh-* with wm-* workflow skills

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Replace the current 4 flat gh-* skills (gh-ingest, gh-plan, gh-implement, gh-commit) with 13 subdirectory-format wm-* skills matching the Knowns SDD workflow (wm-init, wm-research, wm-plan, wm-spec, wm-implement, wm-review, wm-commit, wm-verify, wm-doc, wm-extract, wm-debug, wm-go, wm-template).

Changes needed:
1. Fix scan_skills() in skill.rs to properly parse `wm-*/SKILL.md` subdirectory format (use parent dir name as skill name, not file_stem)
2. Fix frontmatter parsing to read `name:` field (Knowns convention) as primary name source
3. Replace generate_default_skills() to output 13 wm-*/SKILL.md subdirs with SDD workflow content adapted for WM MCP tools (wm_* namespace)
4. Update sync_skills_to() and wm init to generate/clean up the new skill set
5. Remove gh-* references from generation code
6. Update tests to match new skill format
7. Ensure backward compat: existing projects keep their skills, new projects get wm-* skills
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

