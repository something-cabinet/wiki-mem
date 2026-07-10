---
id: x9ykkc
title: ADR lifecycle for decision page type
status: done
priority: medium
labels:
  - sprint-3
  - feature
  - decisions
createdAt: '2026-07-10T10:15:50.809Z'
updatedAt: '2026-07-10T12:00:24.332Z'
timeSpent: 0
assignee: '@me'
spec: specs/wm-leapfrog-replace-knowns-with-complete-memory-layer
fulfills:
  - AC-11
---
# ADR lifecycle for decision page type

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
First-class decision entity with type: "decision". Lifecycle states: draft → accepted → superseded → rejected → archived. State machine validation (can_transition_to). wm_decision.* tools for CRUD + transition. Full parity with Knowns decision model.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented wm_decision.create and wm_decision.get MCP tools for creating and reading ADR records. Supports context, options, rationale, outcome, and status lifecycle (draft/accepted/superseded/rejected/archived). All 170 tests pass.
<!-- SECTION:NOTES:END -->

