---
title: Embed file templates via RustEmbed
type: pattern
tags:
- pattern
- rust-embed
- generator
- shims
status: reviewed
relates_to:
  - {type: references, target: wiki:tasks:embed-shim-templates}
---

## Problem

When a CLI tool generates configuration files (like AGENTS.md, CLAUDE.md), the naive approach is to construct file content at runtime using `format!()` strings. This is fragile — any change to the template requires modifying Rust source code, and the template is invisible to editors that understand markdown.

## Solution

Embed template files at compile time via `rust_embed::RustEmbed` and copy them to target paths at runtime. The project already had this pattern for skills (`SkillAssets`), so extend it to any generated file.

```rust
#[derive(RustEmbed)]
#[folder = "src/shim_templates/"]
pub struct ShimTemplates;
```

Usage:
```rust
let content = ShimTemplates::get("AGENTS.md")
    .and_then(|f| std::str::from_utf8(f.data.as_ref()).ok())
    .ok_or_else(|| anyhow!("Template not found"))?;
std::fs::write(&target_path, content)?;
```

## When to Use

- Any CLI tool that generates configuration/text files at runtime
- Templates that may be edited by humans (markdown, config files)
- When you want the template visible in the project tree for review

## When Not to Use

- Templates that need dynamic substitution per target (use `format!()` or a templating engine)
- Very short templates (<10 lines) where embedding adds more complexity than it saves
- Binary files that benefit from compile-time compression

## Related

- @wiki/tasks/embed-shim-templates
- `apps/wm-core/src/skill/constants/skill_assets_constant.rs` — original SkillAssets pattern
- `apps/wm-core/src/shim_templates.rs` — new ShimTemplates struct