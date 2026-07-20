---
title: Replace div[role="button"] with native button in pages list
type: task
status: todo
priority: medium
---

In pages-view.component.ts, change the clickable page list items from `<div role="button" tabindex="0">` to proper `<button type="button">` elements for native keyboard accessibility.