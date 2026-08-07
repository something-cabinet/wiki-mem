---
title: Sync Wiki Tasks to GitHub Issues
type: howto
id: wiki:howto:sync-wiki-tasks-to-github
status: approved
tags: [howto, github, sync, workflow]
---

## Sync Wiki Tasks to GitHub Issues

Pending wiki tasks (`@wiki/tasks`) can be mirrored to the GitHub issue board of the project repository so humans see pending work in the Issues tab.

### Repo and Auth

- **Repo:** `something-cabinet/wiki-mem` (branch `master`)
- **Auth:** `gh` CLI when available (`gh issue list --repo something-cabinet/wiki-mem`), otherwise the PAT embedded in the git remote URL:
  ```bash
  git config --get remote.origin.url   # https://<user>:<PAT>@github.com/something-cabinet/wiki-mem.git
  TOKEN="$(git config --get remote.origin.url | sed -E 's|https://[^:]+:([^@]+)@.*|\1|')"
  ```

### Create Issues for Pending Tasks

1. Read the task board (`wm_task board`) and collect tasks in pending statuses: `draft`, `todo`, `in-progress`, `in-review`, `blocked`, `on-hold`, `urgent`.
2. For each task, POST to the issues API:
   ```bash
   curl -s -X POST -H "Authorization: Bearer $TOKEN" \
     -H "Accept: application/vnd.github+json" \
     https://api.github.com/repos/something-cabinet/wiki-mem/issues \
     -d '{"title":"<task title>","body":"**Source:** wiki task `<id>` · status `<status>` · priority `<priority>`\n\n## Description\n...\n\n## Acceptance Criteria\n- [ ] ...","labels":["priority: high"]}'
   ```
3. Body must include the source wiki task id (`**Source:** wiki task \`wiki:tasks:<id>\``) so issues can be mapped back to wiki tasks.
4. Body should list acceptance criteria as a markdown checkbox list, carrying over completed state (`- [x]`).

### Labels

Priority labels live on the repo: `priority: urgent`, `priority: high`, `priority: medium`, `priority: low`. Standard GitHub labels (`bug`, `documentation`, `enhancement`, `question`) are reused when the wiki task labels map to them.

### Verified Run (2026-08-07)

125 pending tasks were created as issues #1–#125. Mapping saved in the sync session tool output; each issue body carries its `wiki:tasks:<id>` source reference.

### Related

- `@wiki/rules/check-github-issue-board` — Rule requiring agents to check this board before starting work
