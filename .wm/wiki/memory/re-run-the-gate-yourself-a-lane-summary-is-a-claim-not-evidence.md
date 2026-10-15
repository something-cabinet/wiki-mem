---
title: Re-run the gate yourself — a lane summary is a claim, not evidence
type: memory
tags: [orchestration, verification, workflow, testing]
status: active
---

schema_version: 1
state: |-
  Three times in one session a specialist report said something the artifact did not support:

  1. A function was reported at exactly 20 lines; an independent audit of the same file measured 21.
  2. A CI file-size grep supplied to a lane would have been permanently red with zero violations — its `find -exec` loop body ended in an `&&` list, which returns non-zero for every in-budget file and is propagated through `bash -e`. Only running it revealed that.
  3. A claim that removing the code-db cache would cut opens per query from two to one: it did not. The count was unchanged; only the lock lifetime changed.

  Cheap habit: after every lane, re-run the gate yourself (build, then the tests, then any grep guard) and read the produced artifact rather than the summary. Re-running costs minutes; a silently wrong merge costs hours, and the summaries are usually right — which is exactly why the exceptions are worth catching.

  Related but distinct from @wiki/patterns/verify-tree-before-redispatching-failed-lane: that one covers FAILED lanes that already wrote usable work, this one covers SUCCESSFUL lanes whose claims are wrong.

  Refs: @wiki/patterns/verify-tree-before-redispatching-failed-lane
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
