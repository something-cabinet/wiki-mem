# WIKI-MEM

Canonical repository guidance for agents working in this project.

## Quick Start

```bash
wm init              # Initialize wiki structure
wm mcp               # Start MCP server
wm setup <platform>  # Generate agent config files
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

Run `wm mcp` to start the MCP server. Agents connect automatically when configured.
