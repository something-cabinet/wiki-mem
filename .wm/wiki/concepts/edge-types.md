---
title: Edge Types
type: concept
status: reviewed
tags: [graph, edges, reference]
---

# Edge Types

Canonical reference for all 16 WM edge types. Each edge connects two wiki pages with a directed, typed relationship.

## Reference Table

| Edge | Direction | Description |
|---|---|---|
| `extends` | → | A is a specialization of B. Concept extends concept. |
| `implements` | → | A implements or satisfies B. Task implements spec. |
| `example_of` | → | A is an example or instance of B. |
| `part_of` | → | A is a component or sub-element of B. |
| `relates_to` | ↔ | Generic two-way relationship (semantically weakest). |
| `supports` | → | A provides evidence or support for B. |
| `contradicts` | ↔ | A and B are in conflict or disagree. |
| `supersedes` | → | A replaces B as the authoritative version. |
| `depends_on` | → | A requires B to be completed/understood first. |
| `required_by` | → | A is required by B (inverse of depends_on). |
| `questions` | → | A raises a question that B addresses. |
| `answers` | → | A provides an answer to a question raised by B. |
| `references` | → | A references or cites B (no semantic implication). |
| `similar_to` | ↔ | A and B are conceptually similar but not hierarchically related. |
| `causes` | → | A causes or triggers B. |
| `mitigates` | → | A reduces the impact or likelihood of B. |
| `custom(...)` | → | User-defined edge type for project-specific relations. |

## Usage by Skill

| Skill | Edges used | Purpose |
|---|---|---|
| wm-spec | `answers`, `questions`, `implements`, `depends_on`, `extends`, `part_of`, `references`, `supersedes`, `relates_to` | Connect spec decisions, tasks, and concepts |
| wm-extract | `references`, `extends`, `supersedes`, `example_of` | Link extracted knowledge to sources |
| wm-flow | `implements`, `depends_on`, `required_by` | Traverse spec→task graph for execution |
| wm-research | `extends`, `part_of`, `references`, `similar_to` | Explore concept neighborhoods |
