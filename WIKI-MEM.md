# WIKI-MEM

Canonical repository guidance for agents working in this project.

## Priority: load rules and core docs first

- At the start of every session, before answering, planning or calling any other tool, load all rule pages (`wiki/rules/`) and all core docs (`wiki/core/`). This takes priority over every other step.
- Activate the `wm-init` skill first. It loads the core docs, the rules and memory through the wm MCP tools (`wm_initial`, `wm_page`, `wm_memory`). Do not read `.wm/wiki` with shell commands while the wm tools are available.
- Loading rules and core docs is never skipped, deferred or replaced by a summary from memory. If the wm MCP is unavailable, read `.wm/wiki/rules/*.md` and `.wm/wiki/core/*.md` directly and say that MCP was unavailable.
- Active rules are binding for the whole session. They override default agent behavior, except for safety and system instructions.
- Re-check the rules before finishing a task, before any commit, and after a context reset.
- Critical and core knowledge belongs in `wiki/core/`. Review the core docs after every milestone.

## TL;DR

- Load the core docs and every wiki rule at session start, then obey them.

## Session Start

Read these before doing any work (the `wm-init` skill automates this):

1. **Core docs** — every file under `.wm/wiki/core/`:
   - `core/README` — project overview and quick start
   - `core/ARCHITECTURE` — system design, crates, deployment modes
   - `core/CONVENTIONS` — code, file, and wiki conventions
   - `core/critical-patterns` — highest-cost failures and decisions
2. **Rules** — every active page under `.wm/wiki/rules/` is binding.
3. **Memory** — `wm_memory.list({"layer": "project"})` and `wm_memory.list({"layer": "global"})`.

Read core docs with either form:

```json
wm_page.get({"action": "get", "id": "wiki:core:README"})
wm_doc.get({"action": "get", "path": "core/README"})
```

`wm_page.get` takes `id` (`wiki:core:README`). `wm_doc.get` takes a wiki-relative `path` (`core/README`) and has **no `id` parameter** — passing `id` fails with `missing field 'path'`.

## Critical Rules

- Wiki rules under `.wm/wiki/rules/` are authoritative — load and obey every active rule.
- The `wm-init` skill loads rules at session start (MCP tools first, `.wm/wiki/rules/*.md` file-read fallback); review its session-context "Rules" section before doing work.

## Page Bodies

Every page type is record-bearing: the body after the frontmatter is a typed-decision record, not free prose.

```yaml
schema_version: 1
state: |-
  (the page prose, headings preserved, indented two spaces)
questions:
  - id: <canonical-question-id>
    type: choice
    instructions: <one-line prompt>
    options: [<label>, <label>]
answers: {}
```

- `state` holds the prose verbatim as a literal block scalar.
- `questions` must equal the canonical set for the page type; `answers` keys are a subset (may be empty).
- Full contract: `@wiki/reference/typed-decision-record-schema`.
- Upgrade prose pages and legacy memory with `wm-cli migrate` (`--dry-run` plans without writing).
- `wm_page.get`/`wm_page.update` return a warning when a record-bearing page has a prose body.

## Quick Start

```bash
wm-cli init              # Initialize wiki structure
wm-cli mcp               # Start MCP server
wm-cli setup <platform>  # Generate agent config files
```

## Page Types

| Type | Directory | Purpose |
|------|-----------|---------|
| task | `wiki/tasks/` | Actionable work units with ACs |
| spec | `wiki/specs/` | Requirements and goals |
| concept | `wiki/concepts/` | Domain concepts and architecture |
| pattern | `wiki/patterns/` | Reusable solutions |
| decision | `wiki/decisions/` | ADRs with context and rationale |
| howto | `wiki/howto/` | Step-by-step guides |
| reference | `wiki/reference/` | API docs, config tables |
| core | `wiki/core/` | Project-defining docs |
| rule | `wiki/rules/` | Enforceable project rules |
| memory | `wiki/memory/` | Durable knowledge entries |

## Tool Usage

- All tools prefixed with `wm_` (e.g., `wm_search.query`, `wm_page.get`)
- Call `wm_initial` first for project state
- Search before creating/modifying pages
- Use typed edges (`wm_page.link`) to connect pages

## MCP Server

Run `wm-cli mcp` to start the MCP server. Agents connect automatically when configured.
