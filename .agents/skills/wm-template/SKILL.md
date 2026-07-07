---
name: wm-template
description: List, run, and create code generation templates
---

# Code Templates

**Announce:** "Using wm-template."

**Core principle:** PATTERN ONCE → TEMPLATE → CONSISTENT FOREVER.

## Commands

```json
// List available templates
wm_template.list({})

// Get template details
wm_template.get({ "name": "<name>" })

// Generate from template
wm_template.run({ "name": "<name>", "variables": { "name": "<value>" } })
```

## When to Use Templates

- Scaffolding new modules or components
- Creating standard boilerplate
- Enforcing architectural patterns
- Generating wiki page structures

Templates use Handlebars syntax for variable interpolation.
