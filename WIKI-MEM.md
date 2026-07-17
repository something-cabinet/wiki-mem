# WIKI-MEM

Canonical repository guidance for agents working in this project.

This is the **Wiki Memory Engine** (WM) — a Rust-based local knowledge engine with typed graph, triple-mode search (keyword/semantic/hybrid), MCP integration, Ratatui TUI, and SvelteKit Web UI. Special thanks to [Knowns](https://github.com/knowns-dev/knowns) for the inspiration.

## Table of Contents

- [Source of Truth](#source-of-truth)
- [TL;DR](#tldr)
- [Repo Mental Model](#repo-mental-model)
- [How Agents Should Read This File](#how-agents-should-read-this-file)
- [Tool Selection](#tool-selection)
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


## TL;DR

- Read `WIKI-MEM.md` first.
- Call `wm_project.status` at session start to check project readiness, capabilities, and knowledge counts.
- Use WM MCP tools (`wm_*`) first; fall back to the `wm` CLI when MCP is unavailable.
- Search before reading; read only the sections and docs relevant to the current task.
- Never manually edit WM-managed wiki page markdown.
- Let skills handle detailed workflows; this file is for rules, conventions, and routing.
- Plan before implementation unless the user explicitly overrides that workflow.
- Validate before marking work complete.
- Proactively capture durable memory; do not wait for explicit instruction.
- Do not revert user changes you did not make.

## Repo Mental Model

- WM is a Rust local knowledge engine: typed graph, BM25 + vector search, MCP integration, Ratatui TUI, Angular web UI.
- All wiki pages are markdown files in `.wm/wiki/` — tasks, specs, concepts, patterns, decisions, rules, memory, howto, reference.
- Pages reference each other using `@wiki/{type}/{name}` — e.g., `@wiki/tasks/fix-auth`, `@wiki/concepts/auth`, `@wiki/memory/abc123`.
- `WIKI-MEM.md` defines repo-level operating rules; `.agent/skills/wm-*/` skills define step-by-step execution flows.
- Long guidance should be retrieved by section, not blindly injected in full on every request.
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

- Use `wm_project.status` at session start to check project readiness and available capabilities before acting.
- Use WM MCP tools first for tasks, wiki pages, templates, validation, search, code intelligence, and time tracking.
- Use file reading and search tools for local code and text inspection.
- Use shell commands for git, tests, builds, generators, and other terminal operations.
- Prefer targeted retrieval over loading large files in full.
- Use `wm_search.query` for discovery and quick relevance checks.
- Use `wm_search.retrieve` when a workflow needs structured context with citations and context-pack assembly.
- Prefer structured tool parameters over raw CLI fallback.

### Preferred Tool Matrix

| Category | Tool |
|----------|------|
| WM operations | `wm_*` — tasks, wiki pages, templates, memory, search, validation, time |
| Read file | `read` |
| Find files | `glob` |
| Search content | `grep` |
| Run commands | `bash` (git, builds, tests) |
| Edit files | `edit` / `write` |
| Delegate work | `task` — spawn sub-agents for parallel-safe work |


## Memory Usage

- **Session start:** `wm_memory.list({layer: "project"})` to load accumulated project knowledge.
- **After task:** `wm_memory.add` for reusable patterns, decisions, and conventions.
- **Cross-project:** `wm_memory.promote` to move project knowledge to global (`project → global`).
- Memory complements wiki pages: memory is for fast agent recall, wiki pages are for structured human-readable reference.
- Never duplicate the full wiki page content into memory — store a summary and reference the page with `@doc/<path>`.
- During any skill: if you discover a reusable pattern, decision, convention, or failure, save it with `wm_memory.add`. Capture knowledge as it emerges, don't wait for extraction.
- Proactively save durable memory without waiting for the user to say "save this" when confidence is high.
- Use `project` for repo-specific rules, architecture decisions, conventions, recurring failure patterns, and implementation constraints.
- Use `global` for stable user preferences or workflow rules that should carry across repositories and future sessions.
- Ask the user only when the information appears durable but the correct scope (`project` or `global`) is genuinely ambiguous.
- After any meaningful user instruction, correction, or newly discovered pattern, quickly evaluate whether it should be stored as memory and save it when appropriate.
- If the user states a stable collaboration preference, default to saving it as `global` memory unless they clearly scoped it to this repository only.

## Active Rules

Call `wm_page.list({"type": "rule", "status": "active"})` at session start to load active rules. Rules are strict, non-negotiable constraints that apply to every action. Follow them without exception. If no rules match, continue normally.

## Knowledge Capture

When the user says "remember X" (or equivalent), classify and capture immediately. Do not defer to wm-extract.

| Trigger phrase / context | Type | Action |
|---|---|---|
| `don't` / `never` / `must not` / `avoid` / `rule:` | **Rule** | `rules/<slug>.md` with `type: rule`, category inferred from context |
| `always` / `must` / `rule:` | **Rule** | `rules/<slug>.md` |
| `decided:` / `we chose` / `opted for` / `pick` | **Decision** | `decisions/<slug>.md` with context + rationale + outcome |
| `pattern:` / `when _,` / `here's how to` | **Pattern** | `patterns/<slug>.md` |
| `concept:` / `X is a` / `defines` | **Concept** | `concepts/<slug>.md` |
| `failed:` / `broke because` / `root cause` | **Concept (failure)** | `concepts/<slug>.md` with `tags: [failure]` |

If ambiguous, ask: "Save as rule, decision, pattern, or concept?"

## Critical Rules

- Never manually edit WM-managed task or doc markdown.
- Search first, then read only relevant docs and code.
- Follow `@task-<id>`, `@doc/<path>`, and `@template/<name>` references before acting.
- Use `wm_task.update({appendNotes: ...})` for progress updates; `notes` replaces existing notes and should only be used intentionally.
- Validate before marking work complete.
- Use `.agent/skills/wm-*/` skills for detailed workflow execution instead of duplicating step-by-step process here.
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

## References

- All wiki page references use `@wiki/{type}/{name}` — `@wiki/tasks/fix-auth`, `@wiki/concepts/auth`, `@wiki/memory/abc123`, `@wiki/decisions/use-wire`.
- Template references use `@wiki/templates/{name}`.
- Types map to `.wm/wiki/` subdirectory names: `tasks`, `specs`, `concepts`, `patterns`, `decisions`, `rules`, `memory`, `howto`, `reference`, `notes`.
- Follow references recursively before planning, implementation, or validation work.

## Recommended File Roles

- `WIKI-MEM.md`: canonical repo-level guidance.
- `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `OPENCODE.md` — thin compatibility shims that redirect to `WIKI-MEM.md`.
- `.wm/wiki/`: all wiki page content — tasks, specs, concepts, patterns, decisions, memory, howto, reference.
- `.wm/config.json`: project configuration (embedding, search, permissions, custom edge types).
- `.wm/memory/`: project memory entries (agent-written knowledge fragments).
- `.wm/versions/`: field-level version history for tasks and wiki pages.

## Tool Usage Rules

1. **Prefix**: All WM MCP tools use the `wm_` prefix (e.g., `wm_page`, `wm_task`, `wm_search`)
2. **Initial call**: Always call `wm_project.status` first to get project state and available capabilities
3. **Search before act**: Search the wiki before creating or modifying pages to avoid duplication
4. **Code intelligence**: Use `wm_code` for AST-aware code search, symbol lookup, and dependency analysis — not raw grep

## Common Mistakes

### Notes vs Append Notes

- Use `appendNotes` for progress updates and audit trail entries.
- Use `notes` only when intentionally replacing the task's notes content.

### CLI Pitfalls

- In `wm page create` and `wm task create`, use `--status`, `--priority` flags, not positional args.
- Use `--json` for structured reads consumed by agents, scripts, or workflows (get, list, search, retrieve).
- Use `--plain` for human-facing inspection, quick content reads, and logs.
- Raw task/wiki page IDs are expected where a command asks for an ID value — not mentions or paths.

### Retrieval Pitfalls

- Do not read every doc hoping to find the answer; search first.
- Do not replace discovery-oriented search with retrieve by default; use retrieve only when you need assembled context, citations, or expansion metadata.
- Do not repeatedly list the same tasks or wiki pages if the needed context is already loaded.
- Do not quote large file contents when a concise summary is enough.

### Code Intelligence

- Use `wm_code.symbols` to find definitions and understand code structure.
- Use `wm_code.search` for AST-aware pattern matching across the codebase.
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
