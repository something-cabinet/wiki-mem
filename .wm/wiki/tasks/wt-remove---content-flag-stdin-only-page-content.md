---
title: WT: Remove --content flag, stdin-only page content
type: task
status: todo
priority: high
tags: [spec:wiki-tool-reliability, cli]
---

Remove --content flag from wm-cli page create. Both page create and page update read content from stdin. This fixes multiline breakage naturally. Update existing tests that use --content to pipe stdin instead.