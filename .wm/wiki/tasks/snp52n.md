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

schema_version: 1
state: |-
  # Fix skill parser for subdirectory format + name field

  > **Spec:** `specs/wm-sdd-skills`

  > **Fulfills:** AC-1, AC-2

  > *Imported from Knowns task `snp52n`*

  # Fix skill parser for subdirectory format + name field

  ## Description


  Fix `parse_skill_file()` in skill.rs: (1) Detect subdirectory format (`wm-*/SKILL.md`) and use parent directory name as skill name, not `file_stem()`. (2) Read `name:` frontmatter field as primary name source with fallback to parent dir name. (3) Add `pub fn load_skills_from_embed()` that reads from rust-embed into the SkillEngine. (4) Update `scan()` to also load embedded skills.


  ## Acceptance Criteria
questions:
  - id: work_kind
    type: choice
    instructions: What kind of work is this task?
    options:
    - feature
    - bugfix
    - refactor
    - docs
    - test
    - chore
    - migration
  - id: priority
    type: choice
    instructions: What priority is this task?
    options:
    - low
    - medium
    - high
    - urgent
  - id: needs_spec
    type: noul
    instructions: This task depends on a spec.
  - id: has_ac
    type: noul
    instructions: This task has at least one acceptance criterion.
answers: {}
