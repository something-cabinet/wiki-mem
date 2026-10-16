---
title: "Concurrent agents sharing one working tree can silently wipe uncommitted work"
type: concept
id: "wiki:concepts:concurrent-agent-shared-working-tree-hazard"
status: reviewed
tags: [concept, failure-analysis, orchestration, git, worktree]
relates_to:
  - {type: references, target: wiki:tasks:extract-wm-core-01-finalize-step-0}
---

schema_version: 1
state: |-
  ## What went wrong

  Two agent sessions worked the same repository checkout. One session's subagent ran `git stash -u` (message `fixer-wip-baseline-check`) and reset the tree to HEAD to get a clean test baseline. That stash swept up everything uncommitted in that tree - the other session's in-flight edits, its new untracked pages, and a just-approved spec's decision record - and left the tree at HEAD. The other session had verified its edits seconds earlier, so it reported the change as landed while the tree no longer had it.

  ## Root cause

  A git working tree is a single mutable resource. `git stash -u` captures tracked modifications **and** untracked files, then `reset` moves the tree to HEAD; any concurrent writer in the same checkout loses uncommitted state silently. Nothing warns that a stash belongs to another session, and "the file was there a minute ago" is not evidence it still is.

  ## Recovery

  Everything is recoverable while the stash survives:
  - tracked files: `git show 'stash@{0}:<path>'`
  - untracked files: `git show 'stash@{0}^3:<path>'` (the third parent holds untracked content from `stash -u`)
  - or let the owning session `git stash pop`.

  Copy the recovered files to a directory **outside the repo** immediately, so a later `git stash drop` cannot lose them.

  ## Prevention

  - Do not share one working tree between concurrent agents. Give each session its own `git worktree`; the isolation is cheap and the failure is not.
  - Before assuming a change is yours, check `git log` and `git reflog` - `reflog` shows `reset: moving to HEAD`, and `stash list` names the owning session.
  - Re-read the artifact after any write; never trust a summary that a write landed.
  - If a build needs a clean tree, prefer a fresh worktree over stashing the shared one.

  ## Related

  - @wiki/tasks/extract-wm-core-01-finalize-step-0
  - @wiki/notes/session-handover-2026-10-16
questions:
  - id: kind
    type: choice
    instructions: What kind of concept document is this?
    options:
    - concept
    - failure-analysis
    - research-report
    - reference-note
  - id: category
    type: choice
    instructions: Which domain category does this concept belong to?
    options:
    - architecture
    - search-retrieval
    - graph
    - parser-format
    - mcp-tooling
    - cli
    - storage
    - embeddings
    - web-ui
    - process
  - id: maturity
    type: score
    instructions: How mature is the understanding of this concept?
    levels:
    - raw
    - exploratory
    - established
    - stable
  - id: code_referenced
    type: noul
    instructions: This concept references concrete code.
answers: {}