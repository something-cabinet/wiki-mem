---
title: RuleCategory enum — invalid category silently drops frontmatter
type: memory
tags: [parser, bug, frontmatter, silent-failure]
status: active
---

schema_version: 1
state: |-
  The `RuleCategory` enum in `packages/wm-engine/src/models/page_data/rule_category_model.rs` only had 9 variants. When a rule file used `category: workflow` or `category: quality` in its YAML frontmatter, `serde_yaml` failed to deserialize the entire `Frontmatter` struct. The error was silently swallowed by `Err(_)` in `extract_frontmatter()`, returning `None` for the frontmatter, causing `parse_wiki_page` to default to `PageType::Concept`. Added `Workflow` and `Quality` variants to fix. Also changed the silent `Err(_)` to a `tracing::warn!` so future parsing errors are visible in logs.
questions:
  - id: layer
    type: choice
    instructions: Which memory layer does this entry belong to?
    options:
    - project
    - global
    - session
  - id: store_or_skip
    type: noul
    instructions: This entry is worth storing as durable memory.
  - id: dedup_action
    type: choice
    instructions: How should this entry relate to existing memory?
    options:
    - new
    - merge
    - supersede
    - skip
  - id: confidence
    type: score
    instructions: How confident is the recorded knowledge?
    levels:
    - low
    - medium
    - high
answers: {}
