---
id: wiki:concepts:failure-mcp-task-missing-acceptance-criteria
{}
relates_to:
  - {type: references, target: wiki:tasks:task-398z6o-add-dedicated-wm_taskcreategetupdatedelete-tools}
---
id: wiki:concepts:failure-mcp-task-missing-acceptance-criteria

---
id: wiki:concepts:failure-mcp-task-missing-acceptance-criteria
title: Failure: MCP wm_task.create missing acceptance_criteria field
type: concept
tags: [failure, mcp, task, validation, rust]
---
id: wiki:concepts:failure-mcp-task-missing-acceptance-criteria

## What went wrong

wm_task.create and wm_task.update did not accept an acceptance_criteria parameter. This caused 153 validation errors ("tasks without acceptance criteria") across the entire wiki. Skills that created tasks (wm-plan, wm-flow, wm-review, wm-debug) could not set ACs because the MCP tool simply did not expose the field.

## Root cause

The WmTaskAction enum variants Create and Update in action.rs were missing the acceptance_criteria field. The tool handler had no code path to write ACs into the YAML frontmatter.

## Prevention

- When adding a new field to a page model, add it to ALL tool handlers that touch that page type (Create AND Update AND the frontmatter builder)
- Verify with wm_validate.check after adding new task/create tool support
- Cross-check skill docs against tool API — if a skill describes creating tasks but the tool cannot set ACs, the skill docs are misleading

## Time lost

~5 minutes to fix the tool handler. ~30+ minutes of validation noise that should not have existed.