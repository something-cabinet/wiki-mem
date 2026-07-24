---
name: wm-template
description: List, run, and create code generation templates for consistent boilerplate
---

# Code Templates

**Announce:** "Using wm-template."

**Core principle:** USE TEMPLATES FOR CONSISTENT CODE GENERATION.

## Inputs

- Action: list, get, run, or create
- Template name and variables for generation
- Linked pattern doc, if one exists

## Preflight

- Read the linked doc before running a non-trivial template
- Use dry run before generating real files
- Check whether a template already exists before creating a new one

## Step 1: List Templates

Templates are registered in the template system. List available templates:

```json
wm_template.list()
```

## Step 2: Get Template Details

```json
wm_template.get({"name": "<template-name>"})
```

Check: prompts, `doc:` link, files to generate.

## Step 3: Read Linked Documentation

```json
wm_doc.get({"action": "get", "id": "wiki:<doc-path>"})
```

## Step 4: Run Template

**Always dry run first:**

```json
// Dry run first
wm_template.run({"name": "<template-name>",
  "variables": { "name": "MyComponent"},
  "dryRun": true})

// Then run for real
wm_template.run({"name": "<template-name>",
  "variables": { "name": "MyComponent"},
  "dryRun": false})
```

## Step 5: Create New Template

```json
wm_template.create({"name": "<template-name>",
  "description": "Description",
  "content": "{{#each files}}\n{{content}}\n{{/each}}"})
```

## Template Config

```yaml
name: react-component
description: Create a React component
doc: patterns/react-component

prompts:
  - name: name
    message: Component name?
    validate: required

files:
  - template: ".tsx.hbs"
    destination: "src/components/<name>.tsx"
```

## CRITICAL: Syntax Pitfalls

**NEVER write `$` + triple-brace without proper spacing:**

```
// ❌ WRONG — causes template rendering errors
${ {{{camelCase name}}}

// ✅ CORRECT — add space, use ~ for whitespace control
${ {{~camelCase name~}}}
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
wm_template.run({"name": "new_component",
  "variables": {
    "componentName": "UserProfile",
    "folder": "components/user",
    "withTests": "true"}})
```

## Step 6: Validate (after creating template)

```json
wm_validate.check({"scope": "all"})
```

## Failure Modes

- **Missing linked doc** → say so and inspect the template directly
- **Dry run looks wrong** → stop and fix the template before real generation
- **New template overlaps an existing one** → prefer update or consolidation

## Checklist

- [ ] Template variables provided match expectations
- [ ] Read linked documentation (if applicable)
- [ ] Dry run performed before real generation
- [ ] Generated output reviewed
- [ ] Template creation based on repeated patterns
- [ ] **Validated (if created new template)**
- [ ] Template documented as wiki page under `patterns/`

## Red Flags

- Using templates without checking variables — wrong values produce broken output
- Over-templating — not everything needs a template, just what repeats
- Template patterns without clear descriptions — others won't know what they do
- Skipping output review — templates can produce incorrect code with bad inputs
- Missing linked doc reference in template
- `$` + triple-brace syntax error (use `${ {{~var~}}}` pattern)

## Final Response Contract

All built-in skills in scope must end with the same user-facing information order: `wm-init`, `wm-spec`, `wm-plan`, `wm-research`, `wm-implement`, `wm-verify`, `wm-doc`, `wm-template`, `wm-extract`, and `wm-commit`.

Required order for the final user-facing response:

1. Goal/result - state what was accomplished.
2. Key details - include the most important supporting context, refs, assumptions, or validation.
3. Next action - recommend a concrete follow-up command only when a natural handoff exists.

Keep this concise for CLI use. Skill-specific content may extend the key-details section, but must not replace or reorder the shared structure.

Out of scope: explaining, syncing, or generating `.claude/skills/*`. Runtime auto-sync already handles platform copies, so this skill source only defines the built-in output contract.

For `wm-template`, the key details should cover:
- template used/created, variables provided, output location

## Related Skills

- `/wm-implement <task-id>` — Use template output in implementation
- `/wm-extract` — Extract new template from repeated pattern
- `/wm-commit` — Commit template additions


## Next Step Suggestion

```
/wm-implement <task-id>   — Use template output in implementation
/wm-extract               — Extract new template from repeated pattern
/wm-commit                — Commit template additions
```
