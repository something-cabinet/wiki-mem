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

# Sync WriteChannel: replace async channel with direct fs::write

> *Imported from Knowns task `u6kgab`*

# Sync WriteChannel: replace async channel with direct fs::write

## Description


P0 from code review. page::create_page() and page::update_page() were routing writes through a tokio async WriteChannel. The fire-and-forget semantic meant files weren't on disk when wm_index.rebuild scanned the directory, creating a race condition. Tests used sleep(200ms) to work around it.

Done: replaced with direct std::fs::write() in both functions. WriteOp::Flush variant was attempted but deadlocked (blocking tokio worker thread). Direct sync writes are correct for single-user tools.


## Acceptance Criteria
