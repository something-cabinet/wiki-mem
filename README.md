# Wiki Memory Engine (wm)

**Project memory for AI coding agents — plain markdown files, a typed knowledge graph, and an MCP server.**

`wm` turns a folder of markdown wiki pages into a searchable, typed graph and serves it to any MCP-capable agent. Your knowledge stays in `.wm/wiki/` as files you own and can read, diff, and delete. `wm` itself is a single Rust binary — no daemon to babysit, no server to run.

## Why

- **Your agent stops re-learning your project.** Specs, tasks, decisions, and learnings live somewhere it can search, instead of being re-derived from code every session.
- **Plain markdown, git-friendly.** Every page is a file with YAML frontmatter. Review it in a PR like any other change.
- **A typed graph, not a pile of notes.** Pages link to each other with typed edges, so "what depends on this?" is a query, not a grep.
- **Search that works offline.** BM25 keyword search out of the box; optional local ONNX embeddings for hybrid search. No API key, nothing leaves your machine.
- **One binary, nothing to operate.** No daemon, no ports, no Docker. It speaks MCP over stdio.

## Install

Node.js is needed **only to install via npm** — the binary has no runtime dependencies.

```bash
npm install -g @something-cabinet/wm-cli
wm-cli --version
```

Or run it without installing:

```bash
npx @something-cabinet/wm-cli --help
```

## Five-minute setup

**1. Initialize the project.** From the root of any project you want `wm` to remember:

```bash
wm-cli init --no-wizard
```

This creates `.wm/` — a config, an empty wiki, and agent instruction files — using defaults.

Run `wm-cli init` **without** `--no-wizard` for the interactive version, which asks about the semantic-search model, git-tracking mode, and which agent platforms to configure.

**2. Ask it something.**

```bash
wm-cli search query "how does authentication work" --limit 5
```

Ranked wiki pages with snippets and scores. Options: `--mode keyword|semantic|hybrid`, `--type <page-type>`, `--json`. Defaults live in `.wm/config.json`.

**3. Pull a context bundle instead of a result list** — useful when you (or an agent) want the material for a question rather than links to it:

```bash
wm-cli search retrieve "authentication flow" --token-budget 8192
```

**4. Connect your agent.** See the next section.

## Connect your agent

`wm-cli setup <platform>` writes the right MCP config for your client:

```bash
wm-cli setup claude       # .mcp.json
wm-cli setup opencode     # opencode.json
wm-cli setup codex        # .codex/config.toml
wm-cli setup all          # every supported platform
```

Platforms that get an MCP config: `claude`, `opencode`, `codex`, `cursor`, `kiro`, `antigravity`. Also recognised, for agent instruction files: `gemini`, `copilot`, `agents`, `reasonix`.

The Claude / Cursor / Kiro / Antigravity entry looks like this:

```json
{
  "mcpServers": {
    "wm": { "command": "wm-cli", "args": ["mcp"] }
  }
}
```

OpenCode gets the same server nested under an `"mcp"` key with an array command, and Codex is written as TOML — let `setup` generate it rather than copying the block above.

Restart your agent, then just ask it: *"what's in flight?"*, *"how does auth work here?"*, *"start a spec for X"*.

<details>
<summary>What the agent gets (15 skills)</summary>

`wm-init` loads the project docs, rules, and memory at session start. From there:

| Skill | Purpose |
|---|---|
| `wm-research` | Search the wiki and assemble context |
| `wm-spec` | Turn a problem into a reviewed spec |
| `wm-plan` | Turn a spec into tasks with acceptance criteria |
| `wm-implement` | Follow the plan, check ACs, update the wiki |
| `wm-review` | Multi-perspective review with severity findings |
| `wm-verify` | Verify work against the spec |
| `wm-flow` | Orchestrate an approved spec end to end |
| `wm-commit` | Validate, then commit in conventional format |
| `wm-extract` | Capture durable learnings after the work |
| `wm-debug`, `wm-doc`, `wm-template`, `wm-validate`, `wm-go` | Supporting workflows |
</details>

## Commands

| Command | What it does |
|---|---|
| `wm-cli init [--no-wizard]` | Initialize `.wm/` in a project |
| `wm-cli mcp` | Serve the tool surface to an agent (stdio) |
| `wm-cli setup <platform>` | Write that platform's agent/MCP config |
| `wm-cli search query\|retrieve\|resolve` | Find pages, assemble context, resolve a query to a page id |
| `wm-cli page …` | Page CRUD, links, record migration |
| `wm-cli task board` | Task board by status |
| `wm-cli graph …` | Neighbours, stats, path, blast radius |
| `wm-cli index rebuild\|embed\|code` | Rebuild search and code indexes |
| `wm-cli lint check` · `wm-cli validate` | Wiki health and reference integrity |
| `wm-cli decide` | Record a typed decision |
| `wm-cli status` · `wm-cli health audit` | Project and engine state |

That is about a third of it. `wm-cli --help` lists all 21 top-level commands; agents should call the `wm_help` MCP tool for the live tool list.

## What `.wm/` looks like

```text
.wm/
  config.json       # project config — search modes, scoring, embedding, permissions
  AGENTS.md         # the agent handbook generated for this project
  wiki/
    index.md        # auto-generated page index
    core/           # docs that define how this project works
    specs/  tasks/  # work items with acceptance criteria
    concepts/  patterns/  decisions/  howto/  reference/
    rules/          # binding project rules, read at session start
    memory/         # durable knowledge entries
    notes/
  sources/          # ingest state for external sources
  versions/         # page and task history
  state/            # derived indexes — safe to delete and rebuild
```

Two things worth knowing up front:

- **`state/` is derived.** It can always be rebuilt from `wiki/` with `wm-cli index rebuild`. Nothing in the wiki depends on it.
- **Pages are records, not free prose.** A page body is a YAML record (`state`, `questions`, `answers`) so pages stay queryable and comparable. `wm-cli page migrate-records --apply` converts older pages (without `--apply` it prints the plan).

## Semantic search (optional)

Off by default, because it needs a ~134 MB model download.

```bash
wm-cli model download bge-small-en-v1.5
wm-cli index embed
wm-cli search query "how does authentication work" --mode hybrid
```

With a model present, `hybrid` fuses BM25 and cosine similarity. Without one, search falls back to keyword-only BM25 and reports that it is degraded rather than failing.

## Requirements

- macOS, Linux, or Windows
- Node.js — only for the npm install path
- Optional: ~134 MB disk for an embedding model, stored under `~/.wm/models`

## Docs

- `.wm/wiki/core/` — the project's own documentation, written and searched with `wm` itself: `README`, `ARCHITECTURE`, `CONVENTIONS`, `critical-patterns`
- `WIKI-MEM.md` — the repo-level guidance file agents read first
- `wm-cli --help`, and the `wm_help` tool inside a session

## License

MIT OR Apache-2.0

## Acknowledgments

Built based on [Knowns](https://github.com/knowns-dev/knowns). We took their concepts as a starting point and made our own changes to suit our own needs.
