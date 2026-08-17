---
title: Frontmatter scalar quoting
type: memory
tags: [frontmatter, yaml, task-store]
status: active
---

Frontmatter written via raw format! without quoting breaks YAML when values start with [ or contain : or backslashes — tasks become invisible ("task not found"). Quote user-supplied scalars (title, ACs) or write through yaml_helper / a YAML-aware serializer. Full reference: @wiki/patterns/line-based-frontmatter-editing