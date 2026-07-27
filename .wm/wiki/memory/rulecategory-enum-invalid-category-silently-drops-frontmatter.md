---
title: RuleCategory enum — invalid category silently drops frontmatter
type: memory
tags: [parser, bug, frontmatter, silent-failure]
status: active
---

The `RuleCategory` enum in `packages/wm-engine/src/models/page_data/rule_category_model.rs` only had 9 variants. When a rule file used `category: workflow` or `category: quality` in its YAML frontmatter, `serde_yaml` failed to deserialize the entire `Frontmatter` struct. The error was silently swallowed by `Err(_)` in `extract_frontmatter()`, returning `None` for the frontmatter, causing `parse_wiki_page` to default to `PageType::Concept`. Added `Workflow` and `Quality` variants to fix. Also changed the silent `Err(_)` to a `tracing::warn!` so future parsing errors are visible in logs.