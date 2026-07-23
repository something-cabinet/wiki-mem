---
title: GFX: Instantiate ResizeObserver in canvas directive
type: task
status: todo
priority: medium
tags: [spec:graph-ui-fix, bug]
---

canvas-graph.directive.ts declares ResizeObserver at line 37 but never instantiates it. Add new ResizeObserver(() => this.resize()) in ngAfterViewInit, observe canvas.parentElement. Remove the manual this.resize() call at line 63.