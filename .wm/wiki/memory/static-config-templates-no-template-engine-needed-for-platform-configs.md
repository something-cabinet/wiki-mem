---
title: Static config templates — no template engine needed for platform configs
type: memory
tags: [decision, platform, config, knowns]
status: active
---

Platform config templates (opencode.json, .mcp.json, etc.) are static files with "wm-cli" hardcoded — no placeholder substitution needed since wm-cli is on PATH. Don't reach for a template engine just because Knowns uses Handlebars: their Handlebars is for code generation (knowns template run), not platform config. Reference: @wiki/decisions/static-config-templates-no-substitution