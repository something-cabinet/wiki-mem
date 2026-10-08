---
title: GitHub issue board mirrors pending wiki tasks
type: memory
tags: [github, sync, workflow, issue-board]
status: active
---

schema_version: 1
state: |-
  Pending wiki tasks mirror to the GitHub issue board of something-cabinet/wiki-mem (Issues tab). 125 issues (#1-125) were created on 2026-08-07 from the 125 pending tasks (draft/todo/in-progress). Each issue body carries its source `wiki:tasks:<id>`. PAT is embedded in git remote URL (git config --get remote.origin.url, format https://<user>:<PAT>@github.com/...). Priority labels on the repo: priority: urgent/high/medium/low. Rule: @wiki/rules/check-github-issue-board requires checking this board before starting work. Howto: @wiki/howto/sync-wiki-tasks-to-github.
questions:
  - id: layer
    type: choice
    instructions: Which memory layer does this entry belong to?
    options:
    - project
    - global
    - session
  - id: store_or_skip
    type: noul
    instructions: This entry is worth storing as durable memory.
  - id: dedup_action
    type: choice
    instructions: How should this entry relate to existing memory?
    options:
    - new
    - merge
    - supersede
    - skip
  - id: confidence
    type: score
    instructions: How confident is the recorded knowledge?
    levels:
    - low
    - medium
    - high
answers: {}
