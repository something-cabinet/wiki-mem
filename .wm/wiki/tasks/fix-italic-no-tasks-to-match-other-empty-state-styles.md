---
title: Fix italic 'No tasks' to match other empty state styles
type: task
status: done
priority: low
---

In tasks-view.component.ts:64, the italic `No tasks` text in empty accordion sections uses a different style than other empty states. Align it with the app's empty state pattern.