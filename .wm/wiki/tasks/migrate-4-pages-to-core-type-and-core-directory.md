---
title: Migrate 4 pages to core type and core/ directory
type: task
tags:
- from-spec
- spec:core-page-type
status: done
priority: medium
acceptance_criteria:
- text: '4 pages migrated to .wm/wiki/core/ with type: core'
  checked: false
- text: README type changes from reference to core
  checked: false
- text: CONVENTIONS type changes from reference to core
  checked: false
- text: ARCHITECTURE type changes from reference to core
  checked: false
- text: critical-patterns type changes from pattern to core
  checked: false
- text: Cross-references in other pages updated
  checked: false
- text: wm_validate passes after migration
  checked: false
---

Create .wm/wiki/core/ directory. Move README, CONVENTIONS, ARCHITECTURE, and critical-patterns pages to core/ and update their frontmatter type to 'core'. Update cross-references.