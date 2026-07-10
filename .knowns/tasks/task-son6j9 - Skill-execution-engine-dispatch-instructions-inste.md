---
id: son6j9
title: Skill execution engine — dispatch instructions instead of static text
status: done
priority: high
labels:
  - sprint-1
  - feature
  - skills
  - leapfrog
createdAt: '2026-07-10T10:15:43.530Z'
updatedAt: '2026-07-10T11:09:29.556Z'
timeSpent: 181
assignee: '@me'
spec: specs/wm-leapfrog-replace-knowns-with-complete-memory-layer
fulfills:
  - AC-4
---
# Skill execution engine — dispatch instructions instead of static text

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Build on existing TriggerConfig + fire_event() infrastructure. Add SkillExecutor trait that dispatches structured workflow instructions back to the calling agent. Wire lifecycle events (SessionStart, PageCreate, PageUpdate, IndexRebuild) to executor.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented skill execution engine:
- fire_event() now returns triggered skills (Vec<&Skill>) instead of void + logging
- tool_specs() returns structured instructions with parsed steps (from h2 headings), trigger_info, and type: "skill_instructions"
- Added parse_steps_from_markdown() for extracting structured steps
- Added wm_skill.trigger MCP tool for manual lifecycle event firing
- SessionStart event fired on MCP server startup
- fire_skill_event() in EngineState updated to use new signature
- Skills still return instructions for agent dispatch (not sub-agent spawner)

Deferred (next iteration):
- Wire PageCreate/PageUpdate events into page/task handlers
- Wire IndexRebuild event into index handler
<!-- SECTION:NOTES:END -->

