---
title: "Sync WriteChannel: replace async channel with direct fs::write"
type: task
status: done
tags: [review-fix, write-channel]
priority: high
id: u6kgab
acceptance_criteria:
  - text: "page::create_page() and page::update_page() write directly via std::fs::write(), so files are on disk before wm_index.rebuild scans the directory"
  - text: "Async WriteChannel and the deadlocking WriteOp::Flush variant removed from the page write path"
---

schema_version: 1
state: |-
  # Sync WriteChannel: replace async channel with direct fs::write

  > *Imported from Knowns task `u6kgab`*

  # Sync WriteChannel: replace async channel with direct fs::write

  ## Description


  P0 from code review. page::create_page() and page::update_page() were routing writes through a tokio async WriteChannel. The fire-and-forget semantic meant files weren't on disk when wm_index.rebuild scanned the directory, creating a race condition. Tests used sleep(200ms) to work around it.

  Done: replaced with direct std::fs::write() in both functions. WriteOp::Flush variant was attempted but deadlocked (blocking tokio worker thread). Direct sync writes are correct for single-user tools.


  ## Acceptance Criteria
questions:
  - id: work_kind
    type: choice
    instructions: What kind of work is this task?
    options:
    - feature
    - bugfix
    - refactor
    - docs
    - test
    - chore
    - migration
  - id: priority
    type: choice
    instructions: What priority is this task?
    options:
    - low
    - medium
    - high
    - urgent
  - id: needs_spec
    type: noul
    instructions: This task depends on a spec.
  - id: has_ac
    type: noul
    instructions: This task has at least one acceptance criterion.
answers: {}
