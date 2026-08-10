---
title: Frontmatter corruption prevention — line-based YAML edits, quote ids
type: memory
tags: [wiki, yaml, frontmatter, corruption]
status: active
---

Never round-trip a whole YAML frontmatter block through serde_yaml for a field edit — unquoted ids like `652e07` become floats (6520000000.0) and unmodeled fields get dropped. Use line-based helpers (set_yaml_field/remove_yaml_block/ac_set_checked) and always double-quote id. Full: @wiki/patterns/line-based-frontmatter-editing