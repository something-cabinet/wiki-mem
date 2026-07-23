---
title: GFX: Remove d3-force, fjadra-only layout
type: task
status: todo
priority: medium
tags: [spec:graph-ui-fix, cleanup]
---

Remove d3-force dependency from canvas directive and package.json:
- Remove d3-force imports, Simulation types, simulation state
- Remove startSimulation(), updateLinkDistance(), zoomBy(), fitToView() (or reimplement)
- Add public triggerRender() method
- After applyPositions from fjadra, call triggerRender()
- Remove d3-force, d3-zoom, d3-selection from package.json deps