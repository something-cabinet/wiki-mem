---
title: GitHub issue board mirrors pending wiki tasks
type: memory
tags: [github, sync, workflow, issue-board]
status: active
---

Pending wiki tasks mirror to the GitHub issue board of something-cabinet/wiki-mem (Issues tab). 125 issues (#1-125) were created on 2026-08-07 from the 125 pending tasks (draft/todo/in-progress). Each issue body carries its source `wiki:tasks:<id>`. PAT is embedded in git remote URL (git config --get remote.origin.url, format https://<user>:<PAT>@github.com/...). Priority labels on the repo: priority: urgent/high/medium/low. Rule: @wiki/rules/check-github-issue-board requires checking this board before starting work. Howto: @wiki/howto/sync-wiki-tasks-to-github.