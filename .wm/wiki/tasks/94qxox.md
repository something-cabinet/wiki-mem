---
title: "Web UI: Dark Mode + Toasts + Polish"
type: task
status: done
tags: [web-ui, sveltekit]
priority: low
id: 94qxox
acceptance_criteria:
  - text: "Dark mode works via a prefers-color-scheme media query plus a manual toggle in the nav"
  - text: "console.error calls are replaced with on-screen toasts for errors and successes"
  - text: "vis-network is lazy-loaded (no longer a 514KB chunk on every page), and the sources page exposes reprocess/delete actions"
---

schema_version: 1
state: |-
  # Web UI: Dark Mode + Toasts + Polish

  > *Imported from Knowns task `94qxox`*

  # Web UI: Dark Mode + Toasts + Polish

  ## Description


  (1) Dark mode — CSS variables already ready, add prefers-color-scheme media query + manual toggle in nav, (2) Toast/notification system — replace console.error with on-screen toasts for errors/successes, (3) Lazy-load vis-network (514KB chunk currently loaded on every page), (4) Source management actions (reprocess, delete) in sources page.


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
