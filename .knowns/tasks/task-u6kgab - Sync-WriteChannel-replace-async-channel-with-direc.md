---
id: u6kgab
title: 'Sync WriteChannel: replace async channel with direct fs::write'
status: done
priority: high
labels:
  - review-fix
  - write-channel
createdAt: '2026-07-07T08:50:56.198Z'
updatedAt: '2026-07-07T08:50:56.198Z'
timeSpent: 0
---
# Sync WriteChannel: replace async channel with direct fs::write

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
P0 from code review. page::create_page() and page::update_page() were routing writes through a tokio async WriteChannel. The fire-and-forget semantic meant files weren't on disk when wm_index.rebuild scanned the directory, creating a race condition. Tests used sleep(200ms) to work around it.

Done: replaced with direct std::fs::write() in both functions. WriteOp::Flush variant was attempted but deadlocked (blocking tokio worker thread). Direct sync writes are correct for single-user tools.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

