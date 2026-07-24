---
id: wiki:specs:template-prompt-system
title: Template Prompt System
type: spec
status: draft
tags: [templates, knowns-parity, scaffolding]
---
id: wiki:specs:template-prompt-system

## Overview

Extend WM's template system from simple string interpolation to full code scaffolding with prompts, actions, and file generation.

## Locked Decisions

- D14: Add prompt system in scope
- D18: Full action set: add, addMany, modify, append
- D19: 4 prompt types: text, select, confirm, multiselect
- D20: Destination paths support template variable substitution
- Format: YAML `_template.yaml` companion files alongside `.hbs` templates in a directory

## Requirements

### FR-1: _template.yaml format
```yaml
name: go-feature
description: Generate a Go feature scaffold
doc: patterns/go-feature
destination: src/

prompts:
  - name: name
    type: text
    message: Feature name?
    validate: required
  - name: withServer
    type: confirm
    message: Include HTTP server?
    initial: true
  - name: method
    type: select
    message: Transport method?
    choices:
      - grpc
      - rest
      - both
    initial: rest

actions:
  - type: add
    template: model.go.hbs
    path: "internal/{{snakeCase name}}/model.go"
    skip_if_exists: true
  - type: addMany
    source: "server/"
    path: "internal/{{snakeCase name}}/"
    when: "withServer"
  - type: modify
    path: "cmd/main.go"
    insert_after: "// {{snakeCase name}} routes"
    template: route_import.hbs
  - type: append
    path: "cmd/main.go"
    template: route_init.hbs
```

### FR-2: Template discovery
Templates live in `.wm/templates/{name}/` directories. The `_template.yaml` is the entry point. `wm_template.list` returns both old `.json` templates and new directory templates.

### FR-3: Scaffolding execution
`wm_template.run {name, variables}` resolves prompts first (if interactive), then executes each action: renders `.hbs` with variables, writes/modifies/inserts into destination paths.

## Acceptance Criteria
- [ ] AC-1: `_template.yaml` with prompts+actions parses correctly
- [ ] AC-2: `add` action renders template and writes file
- [ ] AC-3: `addMany` renders and writes multiple files
- [ ] AC-4: `modify` inserts rendered template after anchor line
- [ ] AC-5: `append` adds rendered template to end of file
- [ ] AC-6: `@wiki/templates/{name}` references work
- [ ] AC-7: Old `.json` templates still work
