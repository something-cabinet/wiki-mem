---
title: Add auto-dismiss for search error messages
type: task
status: todo
priority: low
---

In search-view.component.ts, make error messages auto-dismiss after the user starts typing new input (e.g. clear error during debounce). Currently errors persist until next search completes.