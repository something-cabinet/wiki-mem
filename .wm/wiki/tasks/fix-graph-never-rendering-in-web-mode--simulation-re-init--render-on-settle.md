---
title: Fix graph never rendering in web mode — simulation re-init + render-on-settle
type: task
status: cancelled
priority: urgent
tags: [bug, graph, web-ui, urgent]
---

From @designer review C1: canvas-graph.directive.ts has no ngOnChanges — simulation bound to initial empty array, graph blank in web mode. applyPositions() never triggers render. Fix: implement ngOnChanges/signal effect to re-init simulation when data arrives, call render() at end of applyPositions(). Gate canvas with @if (graphNodes.length).