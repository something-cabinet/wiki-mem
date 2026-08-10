---
title: Check Git Remote for Embedded PAT Before Any Work
type: rule
id: wiki:rules:check-git-remote-for-pat
status: active
tags: [rule, git, security, credentials]
---

## Rule: Check Git Remote for Embedded PAT Before Any Work

Before starting any work (implementation, research, task updates, issue operations, CI/deploy changes), check whether the git remote URL embeds a Personal Access Token (PAT). If one is present, flag it, do not rely on it silently, and report it to the user before proceeding with any privileged operation.

### Why

A PAT embedded in the git remote URL (`https://<user>:<token>@github.com/...`) is stored in plaintext in `.git/config`. It is exposed to:

- Anyone with filesystem access to the repository (clone of `.git/`, config leaks, tooling that reads remotes)
- Any process that runs `git remote -v` or parses remotes
- Shell history and process listings when the URL is copied

This is exactly what happened in the wiki-mem reconciliation: the machine-scoped PAT from `git config remote.origin.url` was used to close GitHub issues because the user's `gh` keyring token (`gho_...`) lacked write permission. The embedded PAT is a broader-privilege credential that should not be silently relied upon.

### Check Procedure

Before starting any work, run:

```bash
git config --get-regexp 'remote\..*\.url'
```

Then inspect the output for an embedded credential pattern:

```bash
# Look for: https://<user>:<token>@host/ or http://<user>:<token>@host/
git config --get-regexp 'remote\..*\.url' | grep -E '://[^:/]+:[^@]+@'
```

Also check local and global config:

```bash
git config --local --list | grep -i 'url.*@'
git config --global --list | grep -i 'url.*@'
```

### Actions on Detection

1. **Do not print or echo the full token** — only confirm its presence and length (`echo "${#TOKEN}"`).
2. **Flag it to the user** before using it for any privileged operation. State explicitly which credential authorized the action (e.g., "closed using the PAT embedded in the git remote URL, not your gh keyring token").
3. **Prefer the user's own credential** (`gh` keyring / credential helper) when it has sufficient permission.
4. **Recommend remediation** — offer to swap the remote to a credential-helper setup (`gh auth setup-git`, `osxkeychain`) and rotate the leaked token. Do not rotate or modify credentials without explicit user approval.
5. If the repo is or may be shared, treat the embedded token as compromised and advise rotation.

### Exceptions

None. If a PAT in the remote is the only usable credential (e.g., CI service accounts, isolated automation runners where `gh` is unavailable), still surface the fact explicitly to the user before relying on it, and note it in the work summary.

### Related

- `@wiki/rules/check-github-issue-board` — uses `git config --get remote.origin.url` as a fallback when `gh` is unavailable; prefer a credential helper over embedded PATs.
