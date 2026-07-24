---
title: Bulk YAML frontmatter rename via sed
type: memory
tags: [pattern, sed, yaml, migration]
status: active
---

For one-time bulk YAML frontmatter field renames across many wiki files, use line-start-anchored sed. This is safe for frontmatter keys but not for content. Always verify with cargo test and wm_index.rebuild after. Full pattern: @wiki/patterns/bulk-yaml-frontmatter-rename