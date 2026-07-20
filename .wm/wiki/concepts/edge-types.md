---
title: Edge Types
type: concept
status: reviewed
tags: [graph, edges, reference]
---

# Edge Types

Canonical reference for all 9 WM edge types. Each edge connects two wiki pages with a directed, typed relationship.

## Reference Table

| Edge | Direction | Description |
|---|---|---|
| `extends` | → | A is a specialization of B. Concept extends concept. |
| `implements` | → | A implements or satisfies B. Task implements spec. |
| `example_of` | → | A is an example or instance of B. |
| `part_of` | → | A is a component or sub-element of B. |
| `relates_to` | ↔ | Generic two-way relationship (semantically weakest). |
| `supersedes` | → | A replaces B as the authoritative version. |
| `depends_on` | → | A requires B to be completed/understood first. |
| `answers` | → | A provides an answer to a question raised by B. |
| `references` | → | A references or cites B (no semantic implication). |
| `custom(...)` | → | User-defined edge type for project-specific relations. |

**Pruned types** (removed 2026-07-20): `supports`, `contradicts`, `required_by`, `questions`, `similar_to`, `causes`, `mitigates`. These had zero usage and overlapping semantics with the remaining types. The lenient parser degrades them to `Custom` gracefully — existing frontmatter with these types continues to parse without errors.

### Inverse-edge policy

Edge types are **directional but not paired**. Instead of defining inverses (`depends_on`/`required_by`, `implements`/`implemented-by`), pick one canonical direction per relationship and traverse the graph opposite when needed. `petgraph` supports incoming edge traversal natively — a reverse direction doesn't need a separate variant. This keeps the taxonomy small and avoids inconsistent registration (e.g., forgetting to register `implemented-by`).

## Usage by Skill

| Skill | Edges used | Purpose |
|---|---|---|
| wm-spec | `answers`, `implements`, `depends_on`, `extends`, `part_of`, `references`, `supersedes`, `relates_to` | Connect spec decisions, tasks, and concepts |
| wm-extract | `references`, `extends`, `supersedes`, `example_of`, `relates_to` | Link extracted knowledge to sources |
| wm-flow | `implements`, `depends_on` | Traverse spec→task graph for execution |
| wm-doc | `answers`, `implements`, `depends_on`, `extends`, `part_of`, `references`, `supersedes`, `example_of`, `relates_to` | Connect docs during creation |
| wm-research | `extends`, `part_of`, `references`, `depends_on`, `relates_to` | Explore concept neighborhoods |
