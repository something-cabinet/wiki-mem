---
id: zfdv25
title: 'Failure: Stale binary after revert breaks tests'
layer: project
category: failure
tags:
  - test
  - build
  - cargo
  - stale-binary
createdAt: '2026-07-09T08:01:47.257Z'
updatedAt: '2026-07-09T08:01:47.257Z'
---

After git reverting tool name changes, cargo test still used stale cached binary (wm-cli.exe). MCP tests spawn the binary directly, not through cargo. Full clean rebuild (cargo clean) needed. Lesson: always rebuild wm-cli after reverting wm-core changes since MCP tests spawn the binary.
