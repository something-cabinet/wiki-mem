---
title: Wiki Memory Engine (wm)
type: core
id: wiki:README
tags:
- project
- overview
- getting-started
status: reviewed
relates_to:
  - {type: references, target: wiki:core:enterprise-grade}
---

# Wiki Memory Engine (wm)

A local knowledge graph engine for AI-assisted project management. **wm** indexes markdown wiki pages into a typed directed graph with BM25 search, optional semantic search via ONNX embeddings, and a full CLI/TUI for interactive use.

## Quick Start

```bash
# Initialize a project
wm init

# Start the MCP server (for AI agents)
wm mcp

# Or use the interactive TUI
wm
```

## Setup Workflow

1. **Initialize**: `wm init` — interactive wizard (model selection, git tracking, platform configs)
2. **Generate Platform Config**: `wm setup opencode` — MCP config + skill sync
3. **Create Pages**: Markdown files under `.wm/wiki/` with YAML frontmatter
4. **Semantic Search (optional)**: `wm model download bge-small-en-v1.5` + `wm index embed`
5. **Agent Integration**: Connect via MCP over stdio — auto-generated `.mcp.json`

## Project Structure

```
.wm/                          # Wiki data directory
  config.json                 # Project configuration
  wiki/                       # Markdown wiki pages
    core/                     # Project-defining docs (conventions, architecture)
    concepts/                 # Domain concepts and architecture
    decisions/                # ADRs with context and rationale
    howto/                    # Step-by-step guides
    patterns/                 # Reusable solutions
    reference/                # API docs, config tables
    specs/                    # Requirements and goals
    tasks/                    # Actionable work units with ACs
  memory/                     # JSON memory entries
  state/                      # Generated state (vectors.bin)
apps/                         # Rust crates
  wm-core/                    # Library — graph engine, search, MCP tools
  wm-cli/                     # CLI binary — clap + Ratatui TUI
  wm-server/                  # HTTP daemon — Axum REST API
  wm-web/                     # Angular SPA frontend
packages/                     # Supporting crates
  wm-engine/                  # Core engine orchestration
  wm-search/                  # BM25 + hybrid search
  wm-embed/                   # ONNX embedding pipeline
  wm-code-intel/              # Code intelligence
  wm-lsp/                     # Language server protocol
  fjadra-wasm/                # Force-directed graph layout (WASM)
  graph-algo-wasm/            # Petgraph BFS algorithms (WASM)
  bm25-rerank-wasm/           # Client-side BM25 re-scoring (WASM)
  md-parse-wasm/              # Frontmatter + markdown parsing (WASM)
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `wm init` | Initialize a new project |
| `wm mcp` | Start MCP server (stdio) |
| `wm serve` | Start HTTP daemon |
| `wm setup <platform>` | Generate platform config |
| `wm search <query>` | Search wiki pages |
| `wm page get/create/list` | Wiki page operations |
| `wm graph neighbors/path` | Graph traversal |
| `wm task board` | Task board grouped by status |
| `wm time start/stop/add` | Time tracking |
| `wm model download/list` | ONNX model management |
| `wm index rebuild/embed` | Rebuild search indexes |
| `wm lint check/fix` | Wiki linting |
| `wm validate check` | Wiki validation |
| `wm` (no args) | Interactive TUI |

## Requirements

- Rust toolchain 1.75+
- Optional: ~134MB disk for ONNX embedding model

## References

- @wiki/core:enterprise-grade — Architecture and scale targets
- @wiki/core:conventions — Code and project conventions
- @wiki/concepts:graph-architecture — Graph model details
- @wiki/concepts:memory-system — Memory layer design
- [Knowns](https://github.com/knowns-dev/knowns) — Inspiration and upstream patterns
