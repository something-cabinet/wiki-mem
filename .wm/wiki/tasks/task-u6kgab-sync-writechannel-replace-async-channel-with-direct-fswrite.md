---
title: Sync WriteChannel: replace async channel with direct fs::write
type: task
status: done
tags: [review-fix, write-channel]
priority: high
knowns_id: u6kgab
---

# Sync WriteChannel: replace async channel with direct fs::write

> *Imported from Knowns task `u6kgab`*

# Sync WriteChannel: replace async channel with direct fs::write

## Description


P0 from code review. page::create_page() and page::update_page() were routing writes through a tokio async WriteChannel. The fire-and-forget semantic meant files weren't on disk when wm_index.rebuild scanned the directory, creating a race condition. Tests used sleep(200ms) to work around it.

Done: replaced with direct std::fs::write() in both functions. WriteOp::Flush variant was attempted but deadlocked (blocking tokio worker thread). Direct sync writes are correct for single-user tools.


## Acceptance Criteria
