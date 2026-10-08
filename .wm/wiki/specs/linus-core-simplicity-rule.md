---
title: 'Spec: Linus-core simplicity rule (no compensating layers)'
type: spec
id: wiki:specs:linus-core-simplicity-rule
status: draft
relates_to:
  - {type: references, target: wiki:tasks:linus-core-simplicity-rule}
functional_requirements:
  - description: 'Rule page exists at .wm/wiki/rules/no-compensating-layers.md (type: rule, status: active, id wiki:rules:no-compensating-layers)'
    id: FR-1
  - description: 'Rule covers: (1) compensating layers → fix the underlying layer; (2) no self-HTTP/token-theater/retry-loops in local single-user tools; (3) honesty of logs, justifications, and exit codes; (4) test quality — no un-failable tests, no per-test daemon spawns, no kill -9 teardown, per-test timeouts; (5) dead things die — ''''remove after diagnosis'''' honored, zero-caller code deleted, superseded decisions marked'
    id: FR-2
  - description: Each requirement carries an rg-based enforcement probe
    id: FR-3
  - description: 'Cross-links: Related section references no-warnings, no-dead-code-clone-scanning, tdd-red-green-refactor, findings-first-task-spec, critical-patterns'
    id: FR-4
general_goals:
  - description: Encode the Linus-core principles distilled from the 2026-08-12 architecture + test roasts as one binding, enforceable rule page
  - description: Give future work an early tripwire against 'layers of stupidity, each compensating the other
non_functional_requirements:
  - description: 'Concise: rule pages are loaded at session start — keep under ~120 lines'
    id: NFR-1
  - description: 'Enforceable: every clause maps to a searchable probe or a review question'
    id: NFR-2
  - description: 'Grounded: cites the 2026-08-12 incident (proxy/daemon/token layers, 5,300 lines deleted) as motivating example, not as anecdote-only guidance'
    id: NFR-3
---

schema_version: 1
state: |-
  ## Approach

  1. Spec (this page) + task page (@wiki/tasks/linus-core-simplicity-rule) — findings-first.
  2. Author `.wm/wiki/rules/no-compensating-layers.md` in the existing rule style (frontmatter + rule + why + requirements + enforcement + exceptions + related).
  3. Validate: `wm lint check` + `wm validate`; confirm the rule page is picked up (type: rule count increases).
questions:
  - id: kind
    type: choice
    instructions: What kind of spec is this?
    options:
    - feature
    - system
    - doc
    - migration
    - experiment
  - id: scope
    type: choice
    instructions: How wide is the scope of this spec?
    options:
    - local
    - component
    - system
    - project-wide
  - id: status_class
    type: choice
    instructions: What lifecycle class is this spec in?
    options:
    - draft
    - reviewed
    - approved
    - superseded
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers: {}