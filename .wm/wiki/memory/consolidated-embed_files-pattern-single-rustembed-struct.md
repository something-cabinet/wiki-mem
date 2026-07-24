---
title: Consolidated embed_files pattern — single RustEmbed struct
type: memory
tags: [pattern, rust-embed, platform, config]
status: active
---

All embedded template files (shims, skills, configs) live under apps/wm-core/src/embed_files/{shims,skills,configs}/ with a single EmbeddedFiles RustEmbed struct. Config templates are static (no placeholder substitution — wm-cli is always on PATH). Merge logic (write_merged_json) lives in platform_service.rs. Reference: @wiki/patterns/embed-shim-templates