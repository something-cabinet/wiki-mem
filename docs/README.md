# Wiki Memory Engine (wm)

A local knowledge graph engine for AI-assisted project management. **wm** indexes markdown wiki pages into a typed graph with BM25 search, optional semantic search via ONNX embeddings, a full TUI, an MCP server for AI agents, and an Angular web UI.

> **Language:** Rust  
> **License:** MIT  
> **Repo:** https://github.com/something-cabinet/wiki-mem

---

## Install

```bash
npm install -g @something-cabinet/wm-cli
```

Or build from source:

```bash
cargo install wm-cli
```

## Quick Start

```bash
wm init               # Initialize a new project (interactive wizard)
wm mcp                # Start MCP server for AI agents
wm search "query"     # Search the wiki
wm                    # Launch interactive TUI
```

---

## Architecture

```
wm-core/     Library — graph engine, search, MCP tools, template engine
wm-cli/      CLI binary — clap + Ratatui TUI
wm-web/      Web UI — Angular 19 + Axum HTTP server
.wm/
  config.json         # Project configuration
  wiki/               # Markdown wiki pages with YAML frontmatter
    concepts/         # Domain concepts (rank 4)
    decisions/        # ADR lifecycle records (rank 3)
    howto/            # Step-by-step guides (rank 2)
    patterns/         # Reusable solutions (rank 5)
    reference/        # API docs, config tables (rank 1)
    specs/            # Requirements and specs (rank 6)
    tasks/            # Actionable work units (rank 7)
    notes/            # Informal content (rank 0)
  memory/             # Memory entries as wiki pages (project + global)
  templates/          # JSON template files for code generation
  state/
    vectors.db        # ONNX embedding vectors (SQLite, gitignored)
```

### Data Flow

```
User/AI Agent
    │
    ├── MCP (JSON-RPC over stdio) ──→ wm-core ──→ .wm/wiki/
    ├── TUI (Ratatui) ──────────────→ wm-core ──→ .wm/wiki/
    ├── CLI (clap) ─────────────────→ wm-core ──→ .wm/wiki/
    └── Web UI (Angular) ──→ Axum ──→ wm-core ──→ .wm/wiki/
```

### Concurrency Model

```
Readers (search, graph, page get):
    ArcSwap — lock-free atomic snapshots
    Arc<HashMap<String, EmbedVector>>
    Arc<Bm25Index>

Writers (page create/update, memory add):
    RwLock (coarse-grained)
    sequential write channel via mpsc

Staleness detection:
    AtomicBool stale_flag + directory mtime check
    Rebuild triggered on first search after staleness
```

---

## Search System

See [search-scoring-formula.md](search-scoring-formula.md) for the complete formula with LaTeX notation and worked examples.

### Quick Overview

| Mode | Page results | Memory results | Fusion |
|---|---|---|---|
| `keyword` | BM25 + rerank | BM25 (memory index) | RRF |
| `semantic` | Cosine similarity | Cosine similarity | RRF |
| `hybrid` | BM25 + cosine | BM25 + cosine | RRF |

### Parameters

| Parameter | Default | Description |
|---|---|---|
| $k_1$ | 1.2 | Term frequency saturation |
| $b$ | 0.75 | Length normalization |
| RRF $k$ | 60 | Fusion sharpness |
| Title weight | 4.0 | Field weight |
| Body weight | 1.0 | Field weight |
| Recency model | FSRS-6 | Forgetting curve |
| Stability | 7 days | Recency half-life |

### Ranking Dimensions

```
Final sort: score DESC → centrality DESC → page_type_rank DESC → title ASC
```

| Dimension | What it measures | Why it's needed |
|---|---|---|
| **Score** | Textual + semantic relevance | Primary signal (BM25 + cosine + RRF) |
| **Centrality** | Structural importance | Edge-weighted inbound count — a well-connected spec beats an isolated concept at equal score |
| **Page type rank** | Intrinsic artifact importance | Task (7) beats spec (6) beats note (0) at equal score + centrality |
| **Title** | Deterministic stability | Alphabetical tiebreaker |

Centrality is weighted by edge type priority, not raw count:
$$ \text{centrality} = \sum_{e \in \text{inbound}} \text{priority}(\text{type}(e)) $$

| Edge | Priority | Contribution |
|---|---|---|
| `extends` | 10 | 10 per inbound edge |
| `implements` | 9 | 9 per inbound edge |
| `relates_to` | 0 | 0 — structural link, no boost |

---

## Graph Engine

Typed directed graph using `petgraph::StableGraph<WikiPageMeta, EdgeType>`.

### Edge Types (17)

| Edge Type | Priority | Semantic |
|---|---|---|---|
| `extends` | 10 | Specialization / subclass |
| `implements` | 9 | Concrete realization |
| `part_of` | 8 | System composition |
| `supersedes` | 8 | New version replacing old |
| `example_of` | 6 | Concrete illustration |
| `depends_on` | 5 | Prerequisite dependency |
| `answers` | 5 | Response to question |
| `references` | 1 | Weak citation |
| `relates_to` | 0 | Unweighted link |
| `Custom(String)` | 0 | User-defined |

### Traversal

- **Neighbors**: Typed outgoing/incoming edges with topic-aware scoring
- **Path**: BFS shortest path between two pages
- **Subgraph**: BFS neighborhood extraction around a center node
- **Stats**: Node/edge counts by type
- **Cycle detection**: Diagnostic via `petgraph::algo::is_cyclic_directed`

### Atomicity

Graph state is managed via `ArcSwap<GraphSnapshot>` — readers hold an `Arc` to the previous snapshot and never block. Writes build a new graph in background then atomically swap.

---

## Memory System

Three layers:

| Layer | Storage | Scope | Persistence |
|---|---|---|---|
| `project` | `.wm/wiki/memory/*.md` | Project directory | Disk |
| `global` | `~/.wm/wiki/memory/*.md` | Home directory | Disk |
| `session` | `DashMap<String, MemoryEntry>` | MCP server process | None (ephemeral) |

Session memory evicts by FSRS-6 forgetting curve at capacity (1000 entries).

Memory entries are searchable via BM25 (keyword) + cosine similarity (semantic, when ONNX model loaded).

---

## Page Types

| Type | Rank | Directory | Purpose |
|---|---|---|---|
| `task` | 7 | `tasks/` | Actionable work units with ACs |
| `spec` | 6 | `specs/` | Requirements and specification |
| `pattern` | 5 | `patterns/` | Reusable solutions |
| `concept` | 4 | `concepts/` | Domain concepts |
| `decision` | 3 | `decisions/` | ADR lifecycle records |
| `howto` | 2 | `howto/` | Step-by-step guides |
| `reference` | 1 | `reference/` | API/config reference |
| `note` | 0 | `notes/` | Informal content |

Type is inferred from the first path segment:
```rust
"tasks" → task, "specs" → spec, "concepts" → concept,
"patterns" → pattern, "decisions" → decision,
"howto" → howto, "reference" → reference,
"notes" → note,  _ → concept (fallback)
```

---

## Template Engine

Handlebars-style rendering (`wm_template.run`):

| Feature | Support |
|---|---|
| `{{variable}}` | ✅ Simple substitution |
| `{{#if var}}...{{else}}...{{/if}}` | ✅ Conditional blocks |
| `{{#unless var}}...{{/unless}}` | ✅ Inverted conditional |
| `{{#each list}}...{{/each}}` | ✅ Array/object iteration |
| `{{pascalCase name}}` | ✅ 7 case helpers |
| `{{@template/name key=val}}` | ✅ Reference resolution (depth limit: 10) |
| Dot notation | ✅ `{{user.name}}` |
| Cycles | ✅ Detected via depth limit |

Case helpers: `pascalCase`, `camelCase`, `kebabCase`, `snakeCase`, `upperCase`, `lowerCase`, `startCase`.

---

## Reference Resolution

Inline @reference parsing in body text:

| Syntax | Resolves to |
|---|---|
| `@wiki/tasks/<name>` | Task page |
| `@wiki/specs/<name>` | Spec page |
| `@wiki/concepts/<name>` | Concept page |
| `@wiki/patterns/<name>` | Pattern page |
| `@wiki/decisions/<name>` | Decision page |
| `@wiki/memory/<id>` | Memory entry (project or session) |
| `@wiki/reference/<name>` | Reference page |
| `@wiki/howto/<name>` | How-to guide |
| `@wiki/core/<name>` | Core page (README, ARCHITECTURE, CONVENTIONS) |

References inside code blocks (` ``` `) are skipped. Security: path traversal prevented via `canonicalize()` containment check.

---

## Skill System

15 embedded skills with lifecycle event triggers:

| Skill | Purpose |
|---|---|
| `wm-init` | Session initialization — load docs, learnings, memory |
| `wm-plan` | Generate tasks from spec or create new task |
| `wm-implement` | Follow plan, implement, verify |
| `wm-review` | Multi-perspective code review (5 perspectives) |
| `wm-commit` | Wiki validation + conventional commit |
| `wm-extract` | Extract patterns/decisions/failures |
| `wm-flow` | Spec orchestration with parallel sub-agents |
| `wm-doc` | Create/update wiki documentation |
| `wm-research` | Search, graph exploration, codebase analysis |
| `wm-spec` | Spec-Driven Development document creation |
| `wm-go` | Full pipeline: generate → plan → implement → verify → commit |
| `wm-template` | Template management |
| `wm-validate` | Wiki health validation |
| `wm-verify` | SDD coverage verification |
| `wm-debug` | Structured debugging workflow |

**Note:** Skills are instruction documents for AI agents — they return structured steps, not executable code. Use `task` subagents for delegation, not separate sessions.

---

## MCP Tools (49)

| Prefix | Count | Description |
|---|---|---|
| `wm_initial` | 1 | Project state, conventions |
| `wm_search.*` | 3 | query, retrieve, resolve |
| `wm_page.*` | 7 | CRUD + link/unlink |
| `wm_task.*` | 5 | CRUD + board + AC check |
| `wm_memory.*` | 6 | CRUD + promote (3 layers) |
| `wm_graph.*` | 4 | neighbors, path, subgraph, stats |
| `wm_ref.*` | 3 | extract, resolve, resolve_all |
| `wm_decision.*` | 2 | create, get |
| `wm_template.*` | 3 | list, create, run |
| `wm_code.*` | 3 | search, symbols, deps (tree-sitter) |
| `wm_source.*` | 8 | State machine (add→process→complete) |
| `wm_time.*` | 4 | start, stop, add, report |
| `wm_model.*` | 4 | list, download, remove, status |
| `wm_index.*` | 3 | rebuild, embed, status |
| `wm_skill.*` | 16 | 15 skills + trigger |
| `wm_lint.*` | 2 | check, fix |
| `wm_validate.*` | 1 | validate |
| `wm_help` | 1 | Tool documentation |

---

## Web UI

- **Frontend:** Angular 19 standalone + Tailwind CSS + Sim UI components
- **Backend:** Axum HTTP server embedded in Rust binary via `rust-embed`
- **Protocol:** REST + SSE for real-time sync
- **Single binary:** `wm web` starts everything

### Views

| View | Description | Status |
|---|---|---|
| Search | Full-text with type/mode filters | ✅ Functional |
| Graph | Node exploration with typed edges | ⚠️ Read-only text list |
| Tasks | Kanban-style board by status | ✅ Read-only |
| Pages | List + markdown viewer | ✅ Read-only |
| Memory | Layer-switching browser | ✅ Read-only |
| Settings | Engine stats dashboard | ⚠️ No user-configurable settings yet |

### API Routes

| Path | Handler |
|---|---|
| `GET /api/search?q=&type=&mode=` | Search pages + memory |
| `GET /api/graph?id=&depth=&edge_type=` | Graph neighbors |
| `GET /api/pages` | List wiki pages |
| `GET /api/tasks` | List tasks |
| `POST /api/time/start` | Start timer |
| `POST /api/time/stop` | Stop timer |
| `GET /api/memory?layer=` | List memory entries |
| `GET /api/config` | Project config |
| `GET /api/events` | SSE event stream |

---

## CLI Commands

| Command | Description |
|---|---|
| `wm init` | Initialize a new project |
| `wm` | Interactive TUI (Ratatui) |
| `wm mcp` | Start MCP server (stdio) |
| `wm web` | Start web UI (Axum) |
| `wm search <query>` | Search wiki pages |
| `wm page get/list/create` | Wiki page operations |
| `wm graph neighbors/path` | Graph traversal |
| `wm task board` | Task board grouped by status |
| `wm time start/stop/add` | Time tracking |
| `wm model download/list` | ONNX model management |
| `wm index rebuild/embed` | Rebuild search indexes |
| `wm lint check/fix` | Wiki linting |
| `wm validate check` | Wiki validation |
| `wm setup <platform>` | Generate platform configs |

---

## Configuration

`.wm/config.json`:

```json
{
  "project_name": "my-project",
  "embedding": {
    "model_name": "bge-small-en-v1.5"
  },
  "search": {
    "default_mode": "hybrid",
    "default_limit": 20,
    "rrf_k": 60,
    "scoring": {
      "field_weights": { "title": 4.0, "body": 1.0 },
      "recency_model": "fsrs",
      "recency_stability_days": 7,
      "memory_salience_boost": 2.0,
      "memory_salience_clamp": 0.1
    }
  }
}
```

---

## Knowns Migration

WM was built to replace Knowns (a Go project memory system). The `.knowns/` directory has been fully migrated:

| Feature | WM | Knowns |
|---|---|---|
| Language | Rust | Go |
| Graph | Typed edges (17 types, priority-weighted) | Tag-based adjacency |
| Search | BM25 + ONNX + RRF + FSRS-6 recency | BM25 + ONNX + RRF |
| CLI TUI | Ratatui (5-tab) | Bubble Tea |
| Memory | 3 layers (project/global/session) | 2 layers (project/global) |
| Code Intel | Tree-sitter (6 languages) | LSP (removed tree-sitter) |
| Template engine | Handlebars-style (if/each/helpers) | Handlebars-style |
| Web UI | Angular + Axum | SvelteKit + Chi |
| Web server | Embedded single binary | Embedded single binary |
| MCP tools | ~55+ | ~30 |
| Skills | 15 embedded + lifecycle triggers | Text instructions only |
| Page types | 8 (task/spec/pattern/concept/decision/howto/reference/note) | None (flat docs) |
