---
title: 'Failure: Handlebars/.hbs rabbit hole during embed-files spec'
type: concept
id: wiki:concepts:handlebars-hbs-rabbit-hole
relates_to:
  - {type: references, target: wiki:specs:platform-embed-files}
---
id: wiki:concepts:handlebars-hbs-rabbit-hole

---
id: wiki:concepts:handlebars-hbs-rabbit-hole
title: Failure: Handlebars/.hbs rabbit hole during embed-files spec
type: concept
tags: [failure, research, knowns, template]
---
id: wiki:concepts:handlebars-hbs-rabbit-hole


## What went wrong

During the `embed_files` spec exploration, we assumed WM needed a Handlebars-like template engine for platform config files because "Knowns uses Handlebars." We spent roughly 25 minutes discussing template engine choices (`handlebars` vs `tera` vs `mustache`), `.hbs` naming conventions, and placeholder substitution strategies before realizing none of it was needed.

## Root cause

**Conflating Knowns' code generation system with platform config generation.** Knowns uses Handlebars in their `knowns template run` command — a full code scaffolding system with actions (`add`, `addMany`, `modify`, `append`), conditional guards, and variable prompts. This is equivalent to WM's `wm_template.run` MCP tool. It has nothing to do with how platform configs (opencode.json, .mcp.json) are generated. The platform config templates could be static files all along because `wm-cli` is always on `$PATH`.

## Prevention

When researching how another project handles a problem, verify you're comparing the right feature:
- Knowns' Handlebars → Knowns' `knowns template run` → WM's `wm_template.run` (code generation)
- Knowns' platform configs → Built with `map[string]any{}` in Go code → WM should use static embedded files

Ask: "Is this the same category of feature?" before assuming the approach transfers. Platform config generation and code generation are different problems, even if both output files.

Also: default to the simplest approach first. Static files are the default. Only add a template engine when dynamic substitution is proven necessary.

## Time lost

~25 minutes of exploration, question rounds, and GitHub source verification.

## Related
- @wiki/specs/platform-embed-files
- @wiki/decisions/static-config-templates-no-substitution