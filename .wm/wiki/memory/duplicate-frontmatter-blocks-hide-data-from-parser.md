---
title: Duplicate frontmatter blocks hide data from parser
type: memory
tags: [frontmatter, validation, parser, failure]
status: active
---

Wiki pages can carry TWO --- delimited YAML frontmatter blocks (malformed). The parser reads only the FIRST block; ACs or data in the second block are invisible to validation and graph. When a task "already has ACs" but validation still fails, check whether the ACs are in the first block. Full reference: @wiki/concepts/failure-duplicate-frontmatter-blocks-hide-data