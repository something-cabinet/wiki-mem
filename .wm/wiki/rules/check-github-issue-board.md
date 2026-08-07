---
title: Check GitHub Issue Board Before Starting Work
type: rule
id: wiki:rules:check-github-issue-board
status: active
tags: [rule, github, workflow, issue-board]
---

## Rule: Check GitHub Issue Board Before Starting Work

Before beginning any implementation, research, or documentation task, check the GitHub issue board of the project repository. Pending work is tracked as GitHub issues in `something-cabinet/wiki-mem` (Issues tab), and wiki tasks are mirrored to it.

### Why

The GitHub issue board is the canonical external view of pending work. Wiki tasks under `@wiki/tasks` are mirrored to GitHub issues (see `@wiki/howto/sync-wiki-tasks-to-github`), so any agent must check both surfaces to avoid duplicating or missing work.

### Check Procedure

1. **List open issues** — Query the repo issues:
   - `gh issue list --repo something-cabinet/wiki-mem --state open`
   - or via API: `GET /repos/something-cabinet/wiki-mem/issues?state=open`
   - The PAT lives in the git config remote URL (`git config --get remote.origin.url`) when `gh` is unavailable.

2. **Match against wiki tasks** — For each open issue, resolve the `wiki:` task id in its body (`**Source:** wiki task \`wiki:tasks:...\``). Confirm the wiki task still exists and its status is pending (todo/in-progress/draft).

3. **Drift handling**:
   - Issue open but wiki task done/cancelled → close the GitHub issue, or mark it with a `wontfix` label and note the wiki status.
   - Wiki task pending but no issue → create one (title = task title, body includes source id, priority, labels, acceptance criteria).
   - Issue title/labels differ from the wiki task → update the GitHub issue to match.

4. **When picking up work** — Prefer issues that are `priority: urgent` / `priority: high`, then update the issue to reflect that work started (assignee / `in-progress` note referencing the wiki task).

### Exceptions

- Quick checks only — spend at most 30s listing open issues.
- If the same board was confirmed in-sync in the same session, skip re-checking.
- Do not create GitHub issues for wiki tasks that are done, cancelled, or rejected.

### Related

- `@wiki/tasks` — Pending work tracked as wiki tasks
- `@wiki/rules/check-wm-tool-health-before-work` — Tool health checks before starting work
- `@wiki/howto/sync-wiki-tasks-to-github` — How wiki tasks mirror to GitHub issues
