---
---

---

## Context

When building platform config generation for `wm setup <platform>`, the initial approach (based on Knowns' pattern) assumed config templates needed placeholder substitution — replacing `{{command}}` with the binary path. This led to evaluating template engines (Handlebars, tera) and considering `.hbs` file extensions for what would be simple JSON config files.

## Decision

Platform config templates are static files — no placeholder substitution needed. The command is hardcoded as `"wm-cli"` because the binary is always on `$PATH` after installation. Config files are:

- Static JSON files under `embed_files/configs/`
- Loaded from `EmbeddedFiles` at runtime
- Parsed as JSON, merged with existing config via `write_merged_json()`, and written to disk

## Rationale

- `wm-cli` is always on `$PATH` (installed via npm `@something-cabinet/wm-cli` or `cargo install`)
- No template engine dependency needed — `serde_json::from_str()` + `write_merged_json()` is sufficient
- Config templates are trivially small (6-15 lines each) with 0-1 variable fields
- Static files are simpler to understand, edit, and validate than templates
- The merge logic (`write_merged_json`) already handles the only real variable: user-added MCP servers

## Consequences

- Platform config templates live in `embed_files/configs/` as static JSON/TOML
- No `.hbs` suffix, no template engine, no placeholder substitution
- `platform_service.rs` handles the load → parse → merge → write pipeline
- Adding a new platform = drop a config file in `configs/` + add a match arm in `main.rs`

## Related

- @wiki/specs/platform-embed-files
- @wiki/patterns/embed-shim-templates
- @wiki/specs/remove-self-install-flow — distribution via npm/cargo only (2026-07-31)