---
title: Add --page-type-core CSS tokens and graph inference for core/ directory
type: task
tags:
- from-spec
- spec:core-page-type
status: done
priority: high
acceptance_criteria:
- text: --page-type-core CSS token exists in light and dark themes
  checked: false
- text: Files at .wm/wiki/core/*.md auto-resolve to PageType::Core
  checked: false
---

Add --page-type-core CSS custom property in light/dark themes. Add core/ directory inference mapping to PageType::Core in graph.rs.