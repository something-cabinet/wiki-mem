---
name: wm-template
description: List, run, and create code generation templates for consistent boilerplate
---

# Code Templates

**Announce:** "Using wm-template."

**Core principle:** PATTERN ONCE → TEMPLATE → CONSISTENT FOREVER.

## Inputs

- Action: list, get, run, or create
- Template name and variables for generation

## Available Templates

Templates are registered as skills with the `wm_skill.<name>` naming pattern. Invoke a template via:

```json
wm_skill.<template-name>({ "variables": { "name": "<value>" } })
```

## When to Use Templates

| Use Case | Example |
|----------|---------|
| Scaffolding new modules | New React component, service class |
| Standard boilerplate | API endpoint, test file, migration |
| Architectural patterns | Repository pattern, event handler |
| Wiki page structures | Spec template, decision record |
| Config files | CI config, lint config, tool settings |

## Template Variables

Templates use variable interpolation for customization. Example invocation:

```json
wm_wm_skill_new_component({
  "variables": {
    "componentName": "UserProfile",
    "folder": "components/user",
    "withTests": "true"
  }
})
```

## Creating Templates

When you find yourself writing the same structure repeatedly, create a template by documenting the pattern in a wiki page under `patterns/` folder with example usage.

## Checklist

- [ ] Template variables provided match expectations
- [ ] Generated output reviewed
- [ ] Template creation based on repeated patterns
- [ ] Template documented as wiki page under `patterns/`

## Red Flags

- Using templates without checking variables — wrong values produce broken output
- Over-templating — not everything needs a template, just what repeats
- Template patterns without clear descriptions — others won't know what they do
- Skipping output review — templates can produce incorrect code with bad inputs

## Next Step Suggestion

```
/wm-implement <task-id>   — Use template output in implementation
/wm-extract               — Extract new template from repeated pattern
/wm-commit                — Commit template additions
```
