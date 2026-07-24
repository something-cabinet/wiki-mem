---
title: Refactor Pages dialog/URL state management
id: 0d0efb
type: task
status: todo
priority: high
tags: [bug, web-ui, pages, ux]
---

From @designer review C6: (1) Edit/Delete dialogs trapped inside @else branch — opening them destroys view state before user confirms. (2) loadPage() never calls router.navigate — URL doesn't sync. (3) ngOnInit reads route.snapshot once — navigating to different page while already viewing one does nothing. Fix: move dialogs to template root, make loadPage navigate, subscribe to paramMap.
