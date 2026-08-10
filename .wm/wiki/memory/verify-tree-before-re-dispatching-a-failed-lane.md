---
title: Verify tree before re-dispatching a failed lane
type: memory
tags: [orchestration, subagents, workflow]
status: active
---

A failed/cancelled subagent lane often still wrote complete code to the tree. Before re-dispatching, run git status + check the expected artifacts + cargo check — if the work is there and compiles, reconcile instead of redoing (saves minutes-hours and avoids edit conflicts). Full: @wiki/patterns/verify-tree-before-redispatching-failed-lane