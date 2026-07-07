---
id: zhj7eh
title: 'Sync Writes > Async Channels for Single-User Local Tools'
layer: project
category: failure
tags:
  - write-channel
  - async
  - tokio
  - race
createdAt: '2026-07-07T08:07:15.678Z'
updatedAt: '2026-07-07T08:07:15.678Z'
---

For single-user tools, prefer `std::fs::write()` over async write channels through tokio. The async channel's fire-and-forget semantic creates a race between write-returned and file-on-disk. If you must use async writes, ensure a flush barrier that doesn't deadlock the tokio runtime (use `spawn_blocking`). Full reference: @doc/learnings/learning-e2e-test-infrastructure-sync-write-fix
