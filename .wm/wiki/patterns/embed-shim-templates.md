---
id: wiki:patterns:embed-shim-templates
title: Embed file templates via RustEmbed
type: pattern
tags:
- pattern
- rust-embed
- generator
- shims
status: reviewed
relates_to:
  - {type: references, target: wiki:specs:platform-embed-files}
---
id: wiki:patterns:embed-shim-templates

## Problem

When a CLI tool generates configuration files (like AGENTS.md, opencode.json, CLAUDE.md), the naive approach is to construct file content at runtime using `format!()` or `serde_json::json!()` strings. This is fragile — any change to the template requires modifying Rust source code, and the template is invisible to editors that understand markdown or JSON.

## Solution

Embed all generated file templates at compile time via a single `rust_embed::RustEmbed` struct, organized by category under a unified `embed_files/` directory. The binary ships with the templates baked in, and the CLI copies/merges them at runtime.

### Directory structure

```
apps/wm-core/src/embed_files/
├── shims/         # Agent compatibility shims (AGENTS.md, CLAUDE.md, etc.)
├── skills/        # Built-in wm-* skill files (wm-init/SKILL.md, etc.)
└── configs/       # Static platform config templates (opencode.json, etc.)
```

### Single struct

```rust
#[derive(RustEmbed)]
#[folder = "src/embed_files/"]
pub struct EmbeddedFiles;
```

Usage requires full paths:
```rust
// Load a shim file
let content = EmbeddedFiles::get("shims/OPENCODE.md")
    .and_then(|f| std::str::from_utf8(f.data.as_ref()).ok())
    .ok_or_else(|| anyhow!("Template not found"))?;

// Load a skill
let skill = EmbeddedFiles::get("skills/wm-init/SKILL.md");

// Load a config template (static, no substitution needed)
let config = EmbeddedFiles::get("configs/opencode.json");
```

### Platform config templates are static

Platform config files (opencode.json, .mcp.json, .kiro/settings/mcp.json, etc.) are stored as static files in `embed_files/configs/`. Since `wm-cli` is always expected on `$PATH`, the command is hardcoded as `"wm-cli"` — no placeholder substitution or template engine needed. The pipeline is simply: load template → parse JSON → merge with existing config → write.

### Merge logic

`write_merged_json()` reads the existing config file on disk, deserializes it, overlays the embedded template's top-level keys, and re-serializes. This preserves user-added MCP servers — the embedded template only provides the skeleton for the WM entry.

## When to Use

- Any CLI tool that generates configuration/text files at runtime
- Templates that may be edited by humans (markdown, config files)
- When you want the template visible in the project tree for review
- When you have multiple categories of embedded files (shims, skills, configs)

## When Not to Use

- Templates that need dynamic substitution per target (use `format!()` or a templating engine)
- Very short templates (<10 lines) where embedding adds more complexity than it saves
- Binary files that benefit from compile-time compression

## Prefer single struct over multiple

Having multiple RustEmbed structs (`ShimTemplates`, `SkillAssets`) increases maintenance surface and makes it harder to add new file categories. A single `EmbeddedFiles` struct at a well-organized `embed_files/` directory keeps the pattern extensible:

```rust
// To add a new category, just drop files into the right subdirectory:
EmbeddedFiles::get("configs/kiro_mcp.json");
EmbeddedFiles::get("shims/GEMINI.md");
EmbeddedFiles::get("skills/wm-plan/SKILL.md");
```

## Related

- @wiki/tasks/a9a1fb
- @wiki/specs/platform-embed-files
- `apps/wm-core/src/embed_files.rs` — EmbeddedFiles struct
- `apps/wm-core/src/platform_service.rs` — template loading + merge logic