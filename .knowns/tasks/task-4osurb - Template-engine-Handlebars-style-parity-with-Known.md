---
id: 4osurb
title: Template engine — Handlebars-style parity with Knowns
status: done
priority: medium
labels:
  - sprint-3
  - feature
  - templates
createdAt: '2026-07-10T10:15:47.957Z'
updatedAt: '2026-07-10T11:56:26.140Z'
timeSpent: 261
assignee: '@me'
spec: specs/wm-leapfrog-replace-knowns-with-complete-memory-layer
fulfills:
  - AC-6
---
# Template engine — Handlebars-style parity with Knowns

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Full template engine: if/unless/each/with block helpers, 7 case-conversion helpers (camelCase, pascalCase, kebabCase, snakeCase, upperCase, lowerCase, startCase), file operations (add, addMany, modify, append with Unique), dryRun mode, @template reference resolution, error handling with cycle detection.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented Handlebars-style template engine:
- New template_engine.rs module with recursive descent parser
- {{variable}} substitution with dot notation (user.name)
- {{#if}}/{{#unless}} blocks with {{else}} support, literal true/false
- {{#each}} iteration over arrays and object items
- 7 case helpers: pascalCase, camelCase, kebabCase, snakeCase, upperCase, lowerCase, startCase
- {{@template/name key=val}} reference resolution with depth limit (10)
- Integrates with existing wm_template.run MCP tool
- 16 new tests, 170 total tests pass
<!-- SECTION:NOTES:END -->

