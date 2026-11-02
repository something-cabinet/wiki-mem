---
title: Ship the load-rules-first priority in WIKI-MEM.md and the Kiro steering
type: task
id: "wiki:tasks:ship-the-load-rules-first-priority-in-wiki-memmd-and-the-kiro-steering"
status: done
priority: medium
tags: [agent-guidance, kiro, steering]
spec: wiki:specs:ship-load-rules-first-priority-in-wiki-mem-guidance
acceptance_criteria:
  - text: The embedded shims/WIKI-MEM.md opens with the Priority section, before Session Start
    checked: true
  - text: The embedded Kiro steering template carries the Priority paragraph and still includes WIKI-MEM.md
    checked: true
  - text: The repo WIKI-MEM.md and .kiro/steering/wiki-mem.md equal the embedded templates, enforced by a test
    checked: true
  - text: wm-cli setup kiro writes the steering with the Priority paragraph
    checked: true
  - text: eightcap-new-portal WIKI-MEM.md and steering are regenerated from the templates
    checked: true
implementation_notes: |-
  Priority section added to apps/wm-core/src/embed_files/shims/WIKI-MEM.md and the Priority paragraph to steering/wiki-mem.md; the repo copies WIKI-MEM.md and .kiro/steering/wiki-mem.md are copies of the templates. Tests in apps/wm-core/tests/guidance_templates_test.rs: three failed first, two parity tests guard the repo copies. eightcap-new-portal WIKI-MEM.md and .kiro/steering/wiki-mem.md were copied from the templates (uncommitted there; its knowns.md deletion, execution-rules.md and workflow-rules.md are untouched). Follow-up not done: agents --sync does not refresh WIKI-MEM.md or the steering.
  Correction: .kiro/ is git-ignored in this repo, so .kiro/steering/wiki-mem.md here is a local copy and is not committed or tested; a parity test against it would fail on a fresh clone. Criterion 3 is met for WIKI-MEM.md (repo_wiki_mem_equals_the_embedded_template). The steering template is covered by the template test and by setup_kiro_writes_the_steering_with_the_priority_paragraph.
---

schema_version: 1
state: |-
  Adopt the Priority section and steering paragraph that eightcap-new-portal added by hand (issue 145) into the embedded templates and the repo copies, then regenerate the eightcap files. See the spec for what is adopted and what is left out.
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