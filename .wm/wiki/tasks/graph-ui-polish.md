---
title: Graph UI Polish — Node Spacing, Sizing, and Layout
type: task
status: todo
priority: high
tags:
  - web-ui
  - graph
  - ux
---

Fix three graph visualization issues affecting readability: unconnected nodes drift too far, nodes are too small, and the spacing slider only affects connected nodes instead of all nodes.

## Acceptance Criteria

- [ ] Non-connected nodes stay within reasonable viewport distance (not scattered to edges)
- [ ] Default node size increased (currently too small to read labels)
- [ ] Spacing slider controls spacing for ALL nodes (connected + unconnected), not just connected pairs
- [ ] Build passes
