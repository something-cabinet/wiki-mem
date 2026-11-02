---
title: Ship the load-rules-first priority in WIKI-MEM.md and the Kiro steering
type: spec
id: "wiki:specs:ship-load-rules-first-priority-in-wiki-mem-guidance"
status: approved
tags: [agent-guidance, kiro, steering, spec]
acceptance_criteria:
  - text: The embedded shims/WIKI-MEM.md opens with the Priority section, before Session Start
  - text: The embedded Kiro steering template carries the Priority paragraph and still includes WIKI-MEM.md
  - text: The repo WIKI-MEM.md and .kiro/steering/wiki-mem.md equal the embedded templates, enforced by a test
  - text: wm-cli setup kiro writes the steering with the Priority paragraph
  - text: eightcap-new-portal WIKI-MEM.md and steering are regenerated from the templates
---

schema_version: 1
state: |-
  ## Goal

  Every project that uses wm gets the guidance that loading rules and core docs is the first action of a session, the way eightcap-new-portal already has it by hand.

  ## Verified current state

  - eightcap-new-portal has an uncommitted Priority section in WIKI-MEM.md and a Priority paragraph in .kiro/steering/wiki-mem.md (issue 145). The shipped templates do not have them: apps/wm-core/src/embed_files/shims/WIKI-MEM.md, apps/wm-core/src/embed_files/steering/wiki-mem.md, and the repo copies WIKI-MEM.md and .kiro/steering/wiki-mem.md.
  - The eightcap WIKI-MEM.md is the generated version from 2026-07-30: it names wm init and wm mcp instead of wm-cli, and lacks the Session Start list.
  - wm-cli setup kiro overwrites the steering file from the template, so a hand edit is lost on the next run.
  - wm-cli agents --sync refreshes the shims but not WIKI-MEM.md or the steering, so an existing project only gets template changes by copying.

  ## Adopted from eightcap-new-portal

  - Loading the rule pages and core docs is the first action of every session, before answering, planning or any tool call.
  - It is never skipped, deferred or replaced by a summary from memory; when the wm MCP is unavailable, read the rule and core files directly and say so.
  - Active rules are binding and override default agent behaviour, except for safety and system instructions.
  - Re-check the rules before finishing a task, before a commit and after a context reset.
  - Critical and core knowledge belongs in wiki/core, reviewed after every milestone.
  - A short Priority paragraph in the Kiro steering that points to the section.

  ## Not adopted

  - The steering files that include eightcap rule pages (workflow-rules.md) and execution-rules.md. They are that project's rules, not generic guidance.
  - Generating a steering file that includes every active rule. It would put the whole rule set into every prompt, which is the owner's call.

  ## Out of scope

  Making agents --sync refresh WIKI-MEM.md and the steering. It overwrites files that projects may edit, so it needs its own spec.

  ## Verification

  - Tests on the embedded templates fail first, then pass.
  - The repo copies equal the templates.
  - cargo test --workspace stays green.
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
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers:
  kind: doc
  scope: project-wide
  needs_tasks: true
