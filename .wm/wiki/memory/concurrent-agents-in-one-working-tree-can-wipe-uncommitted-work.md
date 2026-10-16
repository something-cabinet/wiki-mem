---
title: Concurrent agents in one working tree can wipe uncommitted work
type: memory
tags: [orchestration, git, worktree, concurrency]
status: active
---

A subagent ran git stash -u + reset in a shared checkout, silently sweeping another session's uncommitted edits and untracked pages. Recover tracked files from stash@{0} and untracked from stash@{0}^3; copy outside the repo at once. Give each concurrent session its own git worktree. Full reference: @wiki/concepts/concurrent-agent-shared-working-tree-hazard