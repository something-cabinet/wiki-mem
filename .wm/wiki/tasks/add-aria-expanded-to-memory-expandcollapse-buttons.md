---
title: Add aria-expanded to memory expand/collapse buttons
type: task
status: done
priority: medium
---

In memory-view.component.ts, add `[attr.aria-expanded]="expanded[e.id]"` to the expand/collapse toggle buttons for accessibility.