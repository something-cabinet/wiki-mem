---
title: GFX: Increase base node radius for label readability
type: task
status: todo
priority: low
tags: [spec:graph-ui-fix, ux]
---

Increase base nodeRadius() in canvas-graph.directive.ts and align the WebGL webgl-graph.renderer.ts formula. Current Canvas: Math.max(14, Math.min(45, sqrt(degree)*7+7)). Target: Math.max(18, Math.min(55, sqrt(degree)*8+10)). Align WebGL with Canvas formula.