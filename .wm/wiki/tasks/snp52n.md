---
title: Fix skill parser for subdirectory format + name field
type: task
status: done
tags: [from-spec, go-mode]
priority: high
id: snp52n
spec: specs/wm-sdd-skills
fulfills: [AC-1, AC-2]
relates_to:
  - {type: implements, target: wiki:specs:wm-sdd-skills}
acceptance_criteria:
  - text: "parse_skill_file() detects the wm-*/SKILL.md subdirectory format and uses the parent directory name as the skill name, not file_stem()"
  - text: "The name: frontmatter field is the primary skill name source with fallback to the parent directory name"
  - text: "load_skills_from_embed() reads skills from rust-embed into the SkillEngine, and scan() also loads embedded skills"
---

# Fix skill parser for subdirectory format + name field

> **Spec:** `specs/wm-sdd-skills`

> **Fulfills:** AC-1, AC-2

> *Imported from Knowns task `snp52n`*

# Fix skill parser for subdirectory format + name field

## Description


Fix `parse_skill_file()` in skill.rs: (1) Detect subdirectory format (`wm-*/SKILL.md`) and use parent directory name as skill name, not `file_stem()`. (2) Read `name:` frontmatter field as primary name source with fallback to parent dir name. (3) Add `pub fn load_skills_from_embed()` that reads from rust-embed into the SkillEngine. (4) Update `scan()` to also load embedded skills.


## Acceptance Criteria
