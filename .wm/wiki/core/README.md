---
title: Wiki Memory Engine (wm)
type: core
tags:
- project
- overview
- getting-started
status: reviewed
---

schema_version: 1
state: |-
  # Wiki Memory Engine (wm)

  A local knowledge graph engine for AI-assisted project management. **wm** indexes markdown wiki pages into a typed directed graph with BM25 search, optional semantic search via ONNX embeddings, and an MCP tool surface plus a CLI.

  ## Quick Start

  ```bash
  # Initialize a project
  wm-cli init

  # Start the MCP server (for AI agents)
  wm-cli mcp
  ```

  ## Setup Workflow

  1. **Initialize**: `wm-cli init` — interactive wizard (model selection, git tracking, platform configs)
  2. **Generate Platform Config**: `wm-cli setup opencode` — MCP config + skill sync
  3. **Create Pages**: Markdown files under `.wm/wiki/` with YAML frontmatter
  4. **Semantic Search (optional)**: `wm-cli model download bge-small-en-v1.5` + `wm-cli index embed`
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
    memory/                     # Legacy JSON memory (migrated by `wm-cli migrate-memory`)
    state/                      # Generated state (code.db, vectors.db)
    versions/                   # Page version history
    sources/                    # Ingested source material
  apps/
    wm-core/                    # Library — graph engine, search, MCP tools
    wm-cli/                     # Binary — clap CLI + in-process MCP server
  packages/
    wm-constants/               # Zero-dependency shared constants
    wm-engine/                  # Core engine orchestration
    wm-search/                  # BM25 + hybrid search
    wm-embed/                   # ONNX embedding pipeline
    wm-code-intel/              # Code intelligence
    wm-lsp/                     # Language server protocol

  > Retired 2026-10-15: `wm-server` and `wm-web` no longer exist, nor do the four WASM crates
  > (`fjadra-wasm`, `graph-algo-wasm`, `bm25-rerank-wasm`, `md-parse-wasm`). The engine is a
  > single-binary, single-root, in-process tool — see @wiki/core:ARCHITECTURE.
  ```

  ## CLI Commands

  | Command | Description |
  |---------|-------------|
  | `wm-cli init` | Initialize a new project |
  | `wm-cli mcp` | Start MCP server (stdio) |
  | `wm-cli setup <platform>` | Generate platform config |
  | `wm-cli search <query>` | Search wiki pages |
  | `wm-cli page get/list/create/update/delete/link/unlink/migrate-records` | Wiki page operations |
  | `wm-cli graph neighbors/path/subgraph/affected/stats/export` | Graph traversal and export |
  | `wm-cli task board` | Task board grouped by status |
  | `wm-cli time start/stop/add` | Time tracking |
  | `wm-cli model download/list` | ONNX model management |
  | `wm-cli index rebuild/embed/code` | Rebuild search and code indexes |
  | `wm-cli lint check/fix` | Wiki linting |
  | `wm-cli validate` | Wiki validation |
  | `wm-cli status` | Engine and index status |
  | `wm-cli log` | Recent log entries |
  | `wm-cli health` | Wiki health audit |
  | `wm-cli config <action>` | Project configuration |
  | `wm-cli agents` | Agent-facing operations |
  | `wm-cli decide <action>` | Decision records |
  | `wm-cli source <action>` | Manage ingested sources |
  | `wm-cli migrate-memory` | Migrate legacy JSON memory |
  | `wm-cli version` | Version |

  ## Requirements

  - Rust toolchain 1.75+
  - Optional: ~134MB disk for ONNX embedding model

  ## References

  - @wiki/conventions/enterprise-grade — Architecture and scale targets
  - @wiki/core:conventions — Code and project conventions
  - @wiki/concepts:graph-architecture — Graph model details
  - @wiki/concepts:memory-system — Memory layer design
  - [Knowns](https://github.com/knowns-dev/knowns) — Inspiration and upstream patterns
questions:
  - id: kind
    type: choice
    instructions: What kind of core document is this?
    options:
    - conventions
    - architecture
    - patterns
    - glossary
    - reference
  - id: audience
    type: choice
    instructions: Who is this core document written for?
    options:
    - human
    - agent
    - both
  - id: mutability
    type: choice
    instructions: How mutable is this core document?
    options:
    - stable
    - evolving
    - frozen
answers: {}
