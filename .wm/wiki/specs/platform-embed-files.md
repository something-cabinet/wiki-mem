---
title: Platform Embed Files Restructure
type: spec
tags:
- spec
- platform
- embed
- refactor
- knowns
status: approved
implementation_notes: '## Related Tasks - @wiki/tasks/d41ec7 — Remove wm_template.run MCP tool and supporting template_engine module (separate cleanup discovered during spec research)'
---

id: wiki:specs:platform-embed-files

## Technical Notes

- New module: `apps/wm-core/src/embed_files.rs` — single RustEmbed struct
- New module: `apps/wm-core/src/platform_service.rs` — template loading, merge logic, config writing
- Config templates are static (no placeholder substitution needed — `wm-cli` is always on PATH)
- Move `write_merged_json()` and `write_toml_config()` from main.rs to `platform_service.rs`
- Update imports in `skill_frontmatter_parser_helper.rs` and `wm-cli/src/main.rs`
- No new dependencies required

## Open Questions

None — all resolved.