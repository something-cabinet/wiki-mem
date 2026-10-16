---
title: X-01 Docs + embedded-asset parity across the three specs
type: task
id: "wiki:tasks:x-01-docs-and-embedded-asset-parity"
status: todo
tags: [from-spec, spec:global-wiki-layer, spec:configurable-wm-root, spec:remove-global-memory-layer, docs, embedded]
acceptance_criteria:
  - text: ARCHITECTURE carries both amendment sentences; CONVENTIONS reflects all three approved specs
  - text: embed_files shims/skills and their repo-root twins are byte-identical, enforced in lint or CI
  - text: The wm-init session-start block is internally consistent across every edit made by the three specs
priority: high
---

schema_version: 1
state: |-
  Two approved specs edit the same shipped surfaces (a third, wiki:specs:global-wiki-layer, was superseded on 2026-10-15), so this must land as one change or the session-start instructions become internally inconsistent.

  - `wiki:core:ARCHITECTURE` — adopt one amendment sentence only, the two-scope composition sentence having been withdrawn with that superseded spec: "State-root and home resolution are single functions with defined failure semantics; no resolution path may fall back to the current working directory."
  - `wiki:core:CONVENTIONS` — memory guidance (Spec A), cascade/shadow/closed-set (global-wiki-layer), pointer model and config grammar (configurable-wm-root).
  - `WIKI-MEM.md` + the six `embed_files/shims/*` + their repo-root twins + `copilot-instructions.md`.
  - The session-start memory/rules block in `embed_files/skills/wm-init/SKILL.md` **and** `.opencode/skills/wm-init/SKILL.md`: Spec A removes the global *memory* load, global-wiki-layer adds the global *wiki* overlay and shadow counts, configurable-wm-root documents the pointer model.
  - Enforce byte-parity between `embed_files/` and the repo-root twins in lint/CI. This drift has bitten the repo twice already and is load-bearing here.
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