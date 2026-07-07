# AGENTS.md — Wiki Memory Engine Agent Handbook

## Wiki Conventions

### 7 Page Types
| Type | Directory | Purpose |
|------|-----------|---------|
| task | `wiki/tasks/` | Actionable units of work with acceptance criteria |
| spec | `wiki/specs/` | Functional/non-functional requirements, goals |
| concept | `wiki/concepts/` | Domain concepts, terminology, architecture |
| pattern | `wiki/patterns/` | Reusable solutions, when-to-use, examples |
| decision | `wiki/decisions/` | ADRs: context, options, rationale, outcome |
| howto | `wiki/howto/` | Step-by-step guides, tutorials |
| reference | `wiki/reference/` | API docs, error codes, configuration tables |

### Frontmatter Schema
Every wiki page starts with YAML frontmatter:
```yaml
---
title: Page Title
type: task|spec|concept|pattern|decision|howto|reference
status: todo|in-progress|done|draft|reviewed|approved
tags: [tag1, tag2]
priority: low|medium|high|urgent
assignee: name
confidence: high|medium|low
---
```
Per-type fields (spec): `functional_requirements`, `non_functional_requirements`, `general_goals`
Per-type fields (decision): `decision.context`, `decision.options`, `decision.rationale`, `decision.outcome`
Per-type fields (task): `acceptance_criteria`, `estimate`, `prerequisites`

## Workflow Instructions

Always follow this sequence for every request:
1. **Search** — Gather relevant context using `wm_search.query`, `wm_search.retrieve`, or `wm_graph.neighbors`
2. **Gather context** — Read full pages with `wm_page.get`; retrieve context packs with `wm_search.retrieve`
3. **Plan** — Create or update task pages with `wm_page.create` / `wm_page.update`; define acceptance criteria
4. **Implement** — Execute the plan; update pages as needed; link related pages with `wm_page.link`

## Tool Usage Rules

1. **Prefix**: All tools use the `wm_` prefix (e.g., `wm_search.query`, `wm_page.get`)
2. **Initial call**: Always call `wm_initial` first to get project state, graph stats, and available search modes
3. **Search before act**: Search the wiki before creating or modifying pages to avoid duplication
4. **Use JSON output**: Prefer JSON mode (`json=true`) for structured responses in automated workflows

## Canonical Workflows

### 1. gh-ingest — Source Ingestion
- Trigger: New source file discovered
- Steps: `wm_source.list(state=pending)` → `wm_source.process` → `wm_source.complete`
- Creates wiki pages from raw source content

### 2. gh-plan — Implementation Planning
- Trigger: Task assigned
- Steps: Search wiki for related specs/patterns → Create plan with acceptance criteria
- Output: Task page with prerequisites, estimate, and acceptance criteria

### 3. gh-implement — Code & Documentation
- Trigger: Plan ready
- Steps: Review plan → Search related patterns/concepts → Implement changes
- Updates: Task status, links to implemented specs/patterns

### 4. gh-commit — Verification & Commit
- Trigger: Implementation complete
- Steps: Validate wiki (`wm_validate.check`) → Lint check (`wm_lint.check`) → Update task status
- Output: Commit message with wiki page references
