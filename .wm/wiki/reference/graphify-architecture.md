---
title: Graphify Architecture Reference
type: reference
id: wiki:reference:graphify-architecture
status: draft
tags: [reference, graphify, code-intel, llm, tree-sitter, edges, architecture]
relates_to:
  - {type: references, target: wiki:tasks:research-graphify-local-llm-usage-for-wm-code-intel-augmentation}
---

## Overview

Graphify (github.com/Graphify-Labs/graphify, v8, 106k stars) is a Python library + AI coding skill that turns any folder of code/docs/papers into a queryable knowledge graph. It supports 30+ languages via tree-sitter, multiple LLM backends for augmented extraction, and exports to HTML/Obsidian/SVG/Neo4j/GraphML/Cypher.

## Pipeline

```
detect() → extract() → build() → cluster() → analyze() → report.generate() → export.to_*()
```

Each stage is its own module; stages communicate via plain Python dicts and NetworkX graphs.

## Local LLM Integration (graphify/llm.py)

### Multi-Backend Architecture

All backends use the OpenAI client library via OpenAI-compat endpoints (except Bedrock/Azure with native SDKs):

| Backend | Default Model | Base URL | Cost |
|---------|--------------|----------|------|
| ollama | qwen2.5-coder:7b | localhost:11434/v1 | Free (local) |
| kimi | kimi-k2.6 | api.moonshot.ai/v1 | $0.74/$4.66 per 1M |
| gemini | gemini-3-flash-preview | generativelanguage.googleapis.com/v1beta/openai/ | $0.50/$3.00 |
| openai | gpt-4.1-mini | api.openai.com/v1 | $0.40/$1.60 |
| deepseek | deepseek-v4-flash | api.deepseek.com | $0.14/$0.28 |
| claude | claude-sonnet-4-6 | api.anthropic.com | $3.00/$15.00 |
| azure | gpt-4o | (AZURE_OPENAI_ENDPOINT) | $2.50/$10.00 |
| bedrock | anthropic.claude-* | (AWS region) | varies |

### Usage Pattern

- LLM is **opt-in** via `graphify extract . --backend ollama` (or any backend)
- Tree-sitter does the heavy lifting first; LLM augments for ambiguous/inferred edges
- File slicing: 20K char cap per file, packed into batches by token count (tiktoken cl100k_base, fallback 4 chars/token)
- Concurrent extraction via ThreadPoolExecutor
- Graceful degradation: tree-sitter extraction works without any LLM

### Ollama Configuration

```bash
# Environment variables
OLLAMA_BASE_URL=http://localhost:11434/v1    # explicit
OLLAMA_HOST=localhost:11434                   # Ollama's own var (normalized)
OLLAMA_MODEL=qwen2.5-coder:7b               # override model
OLLAMA_API_KEY=ollama                        # dummy key (local, no auth needed)
```

Resolution: OLLAMA_BASE_URL (verbatim) > OLLAMA_HOST (normalized: add http://, default port 11434, append /v1) > default.

## Code Intelligence (graphify/extractors/)

### Edge Relation Taxonomy

| Relation | Meaning | Confidence |
|----------|---------|------------|
| imports | Symbol-level import (file → symbol) | EXTRACTED |
| imports_from | File-level import dependency (file → file) | EXTRACTED |
| calls | Direct function/method call | EXTRACTED or INFERRED |
| indirect_call | Dispatch-table/callback call | INFERRED |
| inherits | Class inheritance (class → base class) | EXTRACTED |
| implements | Interface/protocol implementation | EXTRACTED |
| references | Type reference (field, param, return, generic, attribute, value) | EXTRACTED |
| re_exports | Barrel re-export (file → symbol from another file) | EXTRACTED |
| contains | Ownership (file → function/class) | EXTRACTED |
| method | Class → method ownership | EXTRACTED |
| case_of | Enum case ownership | EXTRACTED |
| decorates | Decorator → decorated function/class | EXTRACTED |

### Semantic Reference Contexts

The `references` relation carries a `context` field for typed edges:

```python
REFERENCE_CONTEXTS = frozenset({
    "field",           # class/struct field type
    "parameter_type",  # function parameter type annotation
    "return_type",     # function return type annotation
    "generic_arg",     # generic type argument (List[T] → T)
    "attribute",       # decorator/annotation type
    "value",           # value-position type reference
    "type",            # general type reference
})
```

### Confidence Labels (Provenance)

| Label | Meaning | wiki-mem Equivalent |
|-------|---------|---------------------|
| EXTRACTED | Explicitly stated in source (import, direct call) | Explicit |
| INFERRED | Reasonable deduction (call-graph second pass, co-occurrence) | Derived |
| AMBIGUOUS | Uncertain, flagged for review | Ambiguous |

### Cross-File Resolution Pipeline (graphify/extractors/resolution.py)

1. **Import path resolution**: resolves relative/absolute imports to file paths
2. **tsconfig alias resolution**: reads tsconfig.json paths/baseUrl for TS/JS
3. **Workspace package resolution**: pnpm/npm/yarn workspace globs
4. **Re-export chain tracking**: follows barrel files (index.ts) through re-export chains, stamps Derived provenance
5. **Receiver-type inference**: per-language (Java, C#, Swift, C++, TS, Ruby, Python) — maps local variables to their declared types so `x.method()` resolves to the correct class
6. **Disambiguation**: when multiple candidates exist, uses path-distance heuristic (disambiguate_ambiguous_candidates)

### Language Support (30+ via tree-sitter)

Core (separate tree-sitter grammars in pyproject.toml dependencies):
Python, JavaScript, TypeScript, Go, Rust, Java, Groovy, C, C++, Ruby, C#, Kotlin, Scala, PHP, Swift, Lua, Zig, PowerShell, Elixir, Objective-C, Julia, Verilog, Fortran, Bash, JSON

Optional (separate extras):
Pascal, DreamMaker (BYOND), HCL/Terraform, SQL

### Key Patterns Worth Adopting

1. **Deferred imports**: dynamic `import()` marked `deferred: true` so import-cycle detection ignores them (static-only cycles are the real bugs)
2. **Receiver-type inference**: `const x = new Foo(); x.method()` → resolves `method` call to `Foo` class — enables cross-file member-call edges
3. **Schema validation**: `validate.py` enforces the extraction output schema before `build()` consumes it — catches extractor bugs early
4. **MinHash dedup** (`_minhash.py`): near-duplicate file detection to avoid redundant extraction
5. **Origin-file stamps**: edges carry `origin_file` and `target_file` transient metadata for post-pass disambiguation (stripped before export)
6. **Disambiguation by path distance**: when multiple node IDs collide, prefer the candidate whose file is closest (fewest path segments) to the source

## Graph Analysis

| Function | Input | Output |
|----------|-------|--------|
| `god_nodes(G)` | graph | High-degree nodes (potential coupling problems) |
| `surprising_connections(G)` | graph | Edges between distant communities |
| `find_import_cycles(G)` | graph | Static import cycles (deferred excluded) |
| `graph_diff(G_old, G_new)` | two graphs | Added/removed nodes and edges |
| `suggest_questions(G, communities, labels)` | graph + clustering | Questions a newcomer should ask |

### Community Detection

Uses the Leiden algorithm (graspologic, optional dep) for community clustering — superior to Louvain for resolution and determinism. Falls back to Louvain (networkx built-in) when graspologic unavailable.

## MCP Server (graphify/serve.py)

Exposes the graph as an MCP tool server (stdio or HTTP via Starlette):
- Compatible with Claude Code, OpenCode, Kiro, and other MCP clients
- Serves the built graph from `graphify-out/graph.json`
- Supports both MCP SDK 1.x and 2.x API (runtime detection)

## File Watcher (graphify/watch.py)

- Watches source directories via `watchdog`
- Debounced rebuild (3s default) on file change
- `check_update(path)` reports whether extraction is pending

## Comparison to wiki-mem

| Aspect | Graphify | wiki-mem wm-code-intel |
|--------|----------|------------------------|
| Language | Python | Rust |
| Parser | tree-sitter (30+ langs) | tree-sitter (7 langs) |
| Graph lib | NetworkX | petgraph |
| Storage | JSON files (graphify-out/) | SQLite (code.db) |
| Edge types | 12+ relations with contexts | calls, imports, inherits |
| Resolution | Per-language receiver-type + tsconfig + workspace | Single graph_resolver |
| LLM augmentation | Multi-backend (Ollama default) | None (ONNX embeddings for search only) |
| Clustering | Leiden (graspologic) | None |
| Analysis | god_nodes, cycles, diff, questions | None |
| Exports | HTML, Obsidian, SVG, GraphML, Neo4j, Cypher | Obsidian (wiki graph only) |
| MCP | serve.py (stdio + HTTP) | wm_code.* tools (4 tools) |
| Dedup | MinHash | Content-hash (incremental) |
| Watch | watchdog | notify crate |

## Related

- @wiki/tasks/research-graphify-code-intel-edge-extraction-for-wm-adoption
- @wiki/tasks/research-graphify-local-llm-usage-for-wm-code-intel-augmentation
- @wiki/concepts/edge-types — wiki-mem's current 9 edge types