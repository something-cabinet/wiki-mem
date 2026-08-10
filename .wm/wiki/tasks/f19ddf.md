---
title: 'GFX: Instantiate ResizeObserver in canvas directive'
id: f19ddf
type: task
status: done
priority: medium
tags:
- spec:graph-ui-fix
- bug
acceptance_criteria:
- text: ResizeObserver is instantiated in ngAfterViewInit and observes canvas.parentElement
- text: The manual this.resize() call at line 63 is removed, and the canvas resizes when the sidebar collapses or the window changes
---

canvas-graph.directive.ts declares ResizeObserver at line 37 but never instantiates it. Add new ResizeObserver(() => this.resize()) in ngAfterViewInit, observe canvas.parentElement. Remove the manual this.resize() call at line 63.