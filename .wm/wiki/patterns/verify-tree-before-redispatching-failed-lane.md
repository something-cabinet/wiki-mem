---
title: 'Pattern: Verify tree state before re-dispatching a failed lane'
type: pattern
id: wiki:patterns:verify-tree-before-redispatching-failed-lane
status: draft
tags:
- pattern
- orchestration
- subagents
- workflow
relates_to:
  - {type: references, target: wiki:tasks:c990b6}
---

## Problem

When orchestrating parallel subagent lanes, a lane can fail with "Session error" / "Task cancelled" / timeout — but that often happens AFTER the agent already wrote its code. Re-dispatching the same lane from scratch wastes minutes-to-hours redoing work that is already in the tree (and risks conflicting edits on top of it).

## Solution

Before re-dispatching a failed lane, verify the tree state directly:

1. `git status --short` — check for partial edits/untracked files from the failed run.
2. Check the specific artifacts the lane was supposed to produce (e.g. `rg "fn compact_doc_history"` / the expected new file).
3. If the work is present and compiles (`cargo check`), treat the lane as effectively done — reconcile, mark tasks, verify with the target tests, and do NOT re-run it.

This campaign hit it repeatedly: fix-7/fix-8 (CLI-over-HTTP), fix-14/16/19 (core leftovers, security residual) all errored at the harness/verification stage but had landed complete, coherent code in the working tree — confirmed by direct `git status` + file-existence + `cargo check`, then verified green by the follow-up test run.

## When to Use

- Any delegated lane returns error/timeout/cancelled status.
- A lane's write scope overlaps with other concurrent lanes (partial-edit risk is highest there).

## When Not to Use

- The lane failed at startup with no writes (nothing to check — re-dispatch directly).
- The tree shows incoherent/partial edits that don't compile — then reconcile or revert before re-dispatching.

## Related

- @task-c990b6 (a lane that errored but left verified work)
- @wiki/patterns/refresh-derived-state-at-write-path (sibling reliability pattern)