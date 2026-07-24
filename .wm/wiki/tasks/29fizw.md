---
title: "WM SDD Skills: replace gh-* with wm-* workflow skills"
type: task
status: done
tags: [skills, workflow, sdd, knowns]
priority: high
id: 29fizw
---

# WM SDD Skills: replace gh-* with wm-* workflow skills

> *Imported from Knowns task `29fizw`*

# WM SDD Skills: replace gh-* with wm-* workflow skills

## Description


Replace the current 4 flat gh-* skills (gh-ingest, gh-plan, gh-implement, gh-commit) with 13 subdirectory-format wm-* skills matching the Knowns SDD workflow (wm-init, wm-research, wm-plan, wm-spec, wm-implement, wm-review, wm-commit, wm-verify, wm-doc, wm-extract, wm-debug, wm-go, wm-template).

Changes needed:
1. Fix scan_skills() in skill.rs to properly parse `wm-*/SKILL.md` subdirectory format (use parent dir name as skill name, not file_stem)
2. Fix frontmatter parsing to read `name:` field (Knowns convention) as primary name source
3. Replace generate_default_skills() to output 13 wm-*/SKILL.md subdirs with SDD workflow content adapted for WM MCP tools (wm_* namespace)
4. Update sync_skills_to() and wm init to generate/clean up the new skill set
5. Remove gh-* references from generation code
6. Update tests to match new skill format
7. Ensure backward compat: existing projects keep their skills, new projects get wm-* skills


## Acceptance Criteria
