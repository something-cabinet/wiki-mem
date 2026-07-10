# WIKI-MEM

Canonical repository guidance for agents working in this project.

This is the **Wiki Memory Engine** (WM) — a Rust-based local knowledge engine with typed graph, triple-mode search (keyword/semantic/hybrid), MCP integration, Ratatui TUI, and SvelteKit Web UI. It is a reimplementation and evolution of [Knowns](https://github.com/knowns-dev/knowns).

## Table of Contents

- [Source of Truth](#source-of-truth)
- [TL;DR](#tldr)
- [Repo Mental Model](#repo-mental-model)
- [How Agents Should Read This File](#how-agents-should-read-this-file)
- [Tool Selection](#tool-selection)
- [Wiki Conventions](#wiki-conventions)
- [Workflow Instructions](#workflow-instructions)
- [Canonical Workflows](#canonical-workflows)
- [Memory Usage](#memory-usage)
- [Critical Rules](#critical-rules)
- [Git Safety](#git-safety)
- [Context Retrieval Strategy](#context-retrieval-strategy)
- [Tool Usage Rules](#tool-usage-rules)
- [Common Mistakes](#common-mistakes)
- [Compatibility Pattern](#compatibility-pattern)
- [Maintenance Rules](#maintenance-rules)

## Source of Truth

- `WIKI-MEM.md` is the canonical repo-level guidance file.
- `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `OPENCODE.md`, `.github/copilot-instructions.md`, and `.wm/AGENTS.md` are compatibility shims for runtimes that auto-detect those filenames.
- If guidance appears in multiple places, follow this precedence order:
  1. System instructions
  2. Developer instructions
  3. `WIKI-MEM.md`
  4. Compatibility shim files
  5. Other repository docs
- If a shim file and `WIKI-MEM.md` differ, treat `WIKI-MEM.md` as correct.
- `KNOWNS.md` is a parity reference against the upstream Knowns project. When `KNOWNS.md` and `WIKI-MEM.md` differ, follow `WIKI-MEM.md`.

## TL;DR

- Read `WIKI-MEM.md` first.
- Call `knowns_project({ action: "status" })` at session start to check project readiness, available capabilities, and knowledge counts.
- Use the Knowns MCP tools (`knowns_*`) for tasks, docs, templates, memory, validation, search, code intelligence, and time tracking.
- Search before reading; read only the sections and docs relevant to the current task.
- Never manually edit Knowns-managed task or doc markdown.
- Plan before implementation unless the user explicitly overrides that workflow.
- Validate before marking work complete.
- Use memory tools: `knowns_memory({ action: "list", layer: "project" })` at session start, `knowns_memory({ action: "add" })` after tasks for reusable knowledge.
- Proactively capture durable memory; do not wait for explicit instruction.

## Repo Mental Model

- WM is the project's memory layer for humans and the AI-friendly operating layer for agents.
- The Knowns MCP tools manage tasks, docs, templates, specs, references, and workflow state in one place.
- Tasks and docs may reference each other using `@task-<id>`, `@doc/<path>`, and `@template/<name>`.
- `WIKI-MEM.md` defines repo-level operating rules; `.claude/skills/wm-*/` skills define step-by-step execution flows.
- Long guidance should be retrieved by section, not blindly injected in full on every request.

## How Agents Should Read This File

- Always read `## Source of Truth` and `## TL;DR` first.
- For short or obvious tasks, use the summary sections plus the relevant section only.
- For tool usage questions, read `## Tool Selection` and `## Common Mistakes`.
- For safety-sensitive work, read `## Critical Rules` and `## Git Safety`.
- For large files or docs, read `## Context Retrieval Strategy`.
- For ambiguous requests, search the repo and related docs before asking the user.
- Do not assume the entire file is present in context; retrieve the needed sections when required.

## Tool Selection

- Use `knowns_project({ action: "status" })` at session start to check project readiness and available capabilities before acting.
- Use Knowns MCP tools first for tasks, docs, templates, validation, search, code intelligence, and time tracking.
- Use file reading and search tools for local code and text inspection.
- Use shell commands for git, tests, builds, generators, and other terminal operations.
- Prefer targeted retrieval over loading large files in full.
- Use `knowns_search({ action: "search" })` for discovery and quick relevance checks.
- Use `knowns_search({ action: "retrieve" })` when a workflow needs structured context with citations and context-pack assembly.
- Prefer structured tool parameters over raw CLI fallback.

### Preferred Tool Matrix

| Task | Tool |
|------|------|
| Project status | `knowns_project({ action: "status" })` |
| List/search docs | `knowns_docs({ action: "list" })` / `knowns_search({ action: "search" })` |
| Read doc | `knowns_docs({ action: "get", path: "..." })` |
| Create/update doc | `knowns_docs({ action: "create" })` / `knowns_docs({ action: "update" })` |
| Tasks (CRUD) | `knowns_tasks({ action: "create" })` / `get` / `update` / `delete` |
| Task board | `knowns_tasks({ action: "board" })` |
| Memory | `knowns_memory({ action: "list" })` / `add` / `get` / `promote` |
| Validation | `knowns_validate({ scope: "sdd" })` |
| Code intelligence | `knowns_code({ action: "search" })` / `symbols` / `deps` |
| Templates | `knowns_templates({ action: "list" })` / `run` / `create` |
| Time tracking | `knowns_time({ action: "start" })` / `stop` / `add` / `report` |
| Read file | `read` |
| Find files | `glob` |
| Search content | `grep` |
| Run commands | `bash` (git, builds, tests) |
| Edit files | `edit` / `write` |

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

1. **Search** — Gather relevant context using `knowns_search({ action: "search" })` or `knowns_search({ action: "retrieve" })`
2. **Gather context** — Read full pages with `knowns_docs({ action: "get" })`; retrieve context packs
3. **Plan** — Create or update task pages with `knowns_tasks`; define acceptance criteria
4. **Implement** — Execute the plan; update pages as needed

## Canonical Workflows

### 1. wm-init — Session Initialization
- **Trigger:** Start of new session
- **Steps:** Project status → List docs → Check tasks/board → Load memory → Summarize
- **Output:** Session context with project state, memory, and task overview
- **Tools:** `knowns_project`, `knowns_docs`, `knowns_tasks`, `knowns_memory`

### 2. wm-research — Project Research
- **Trigger:** Need to understand context
- **Steps:** `knowns_search({ action: "search" })` → `knowns_docs({ action: "get" })` → `knowns_code({ action: "graph" })`
- **Output:** Cross-entity context across pages + code + memory

### 3. wm-plan — Task Planning
- **Trigger:** Task assigned
- **Steps:** Search wiki for related specs → Create plan with ACs → Validate
- **Supports:** `--from @doc/<spec>` for spec-wide task generation

### 4. wm-implement — Code & Documentation
- **Trigger:** Plan approved
- **Steps:** Follow plan → Check ACs → Validate (`knowns_validate`) → Track time
- **Tracking:** `knowns_time({ action: "start/stop" })`

### 5. wm-review — Code Review
- **Trigger:** Implementation complete
- **Steps:** Multi-perspective review → Severity findings (P0/P1/P2/P3) → Fix P0/P1

### 6. wm-commit — Verification & Commit
- **Trigger:** Review passed
- **Steps:** Validate (`knowns_validate`) → Conventional commit
- **Note:** Ask user before committing

### 7. wm-extract — Knowledge Extraction
- **Trigger:** Pattern discovered
- **Steps:** Review source → Check duplicates → Create wiki page → Save memory → Promote to critical

### 8. wm-flow — Spec/Task Wave Orchestrator
- **Trigger:** Approved spec with multiple tasks
- **Steps:** Task discovery → Parallel gate → Implementation loop → Review → Verify

## Memory Usage

- **Session start:** `knowns_memory({ action: "list", layer: "project" })` to load accumulated project knowledge.
- **After task:** `knowns_memory({ action: "add" })` for reusable patterns, decisions, and conventions.
- **Cross-project:** `knowns_memory({ action: "promote" })` to move project knowledge to global (`project → global`).
- Memory complements docs: memory is for fast agent recall, docs are for structured human-readable reference.
- Never duplicate the full doc content into memory — store a summary and reference the doc with `@doc/<path>`.
- During any skill: if you discover a reusable pattern, decision, convention, or failure, save it with `knowns_memory({ action: "add", layer: "project" })`. Capture knowledge as it emerges, don't wait for extraction.
- Proactively save durable memory without waiting for the user to say "save this" when confidence is high.
- Use `project` for repo-specific rules, architecture decisions, conventions, recurring failure patterns, and implementation constraints.
- Use `global` for stable user preferences or workflow rules that should carry across repositories and future sessions.
- Ask the user only when the information appears durable but the correct scope (`project` or `global`) is genuinely ambiguous.
- After any meaningful user instruction, correction, or newly discovered pattern, quickly evaluate whether it should be stored as memory and save it when appropriate.
- If the user states a stable collaboration preference, default to saving it as `global` memory unless they clearly scoped it to this repository only.

## Critical Rules

- Never manually edit Knowns-managed task or doc markdown.
- Search first, then read only relevant docs and code.
- Follow `@task-<id>`, `@doc/<path>`, and `@template/<name>` references before acting.
- Use `knowns_tasks({ action: "update", appendNotes: ... })` for progress updates; `notes` replaces existing notes and should only be used intentionally.
- Validate before marking work complete.
- Use `.claude/skills/wm-*/` skills for detailed workflow execution instead of duplicating step-by-step process here.
- Compatibility shim files must stay lightweight and must direct agents back to `WIKI-MEM.md` for behavioral rules instead of restating divergent guidance.

## Git Safety

- Assume the worktree may already contain user changes.
- Never revert or overwrite unrelated user changes unless explicitly requested.
- Avoid destructive git commands unless explicitly requested.
- Do not amend commits unless explicitly requested.
- Do not create commits unless the user explicitly asks for a commit.
- Do not push unless the user explicitly asks for it.

## Context Retrieval Strategy

- Treat `WIKI-MEM.md` as an indexed manual, not a prompt to fully inject every time.
- Read in this order when context is limited:
  1. `## Source of Truth`
  2. `## TL;DR`
  3. The section most relevant to the task
- For large or complex tasks, retrieve additional sections on demand.
- Prefer section headings with stable names so tools can target them precisely.
- If a downstream runtime supports startup loading, preload only the top-level summary and fetch deeper sections lazily.

## Tool Usage Rules

1. **Prefix**: All Knowns MCP tools use the `knowns_` prefix (e.g., `knowns_docs`, `knowns_tasks`, `knowns_search`)
2. **Initial call**: Always call `knowns_project({ action: "status" })` first to get project state and available capabilities
3. **Search before act**: Search the wiki before creating or modifying pages to avoid duplication
4. **Code intelligence**: Use `knowns_code` for AST-aware code search, symbol lookup, and dependency analysis — not raw grep

## Common Mistakes

### Notes vs Append Notes

- Use `appendNotes` for progress updates and audit trail entries.
- Use `notes` only when intentionally replacing the task's notes content.

### Retrieval Pitfalls

- Do not read every doc hoping to find the answer; search first.
- Do not replace discovery-oriented search with retrieve by default; use retrieve only when you need assembled context, citations, or expansion metadata.
- Do not repeatedly list the same tasks or docs if the needed context is already loaded.
- Do not quote large file contents when a concise summary is enough.

### Code Intelligence

- Use `knowns_code({ action: "symbols" })` to find definitions and understand code structure.
- Use `knowns_code({ action: "search" })` for AST-aware pattern matching across the codebase.
- Avoid raw `grep` for structural questions (symbol definitions, references) — the code intelligence tool understands the AST.

## Compatibility Pattern

- Keep shim files short.
- In every shim file, explicitly say that `WIKI-MEM.md` is canonical.
- Preserve the `<!-- WIKI-MEM GUIDELINES START -->` and `<!-- WIKI-MEM GUIDELINES END -->` markers in shim files so tooling can detect and sync them reliably.

## Maintenance Rules

- Update the `wm init` generator when the repository's operational rules change.
- Keep top sections stable so automated loaders can depend on them.
- Prefer adding new sections over bloating the TL;DR.
- Keep workflow details in skills when possible; keep `WIKI-MEM.md` focused on rules, conventions, and routing.
