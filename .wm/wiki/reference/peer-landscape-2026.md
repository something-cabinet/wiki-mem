---
title: Peer Landscape 2026 — Agent Memory, Wiki Graphs & SDD Tooling
type: reference
id: "wiki:reference:peer-landscape-2026"
status: draft
tags: [research, competitive-analysis, knowledge-graph, mcp, sdd]
---

## Scope & Method

Peer research for WM (Wiki Memory Engine): a Rust, local-first, single-binary markdown→typed-graph engine with BM25 + optional ONNX semantic search + graph traversal, an MCP stdio server (`wm_*`), a CLI, SDD workflow skills, and project/global/session memory layers.

Method: three research lanes (2026-10-15) — (1) read-only inventory of prior-art already recorded in this wiki, (2) external survey of the agent-memory / knowledge-graph-MCP ecosystem, (3) external survey of local-first markdown wiki, code-intel, and SDD tooling. External repo identity/activity claims were spot-checked against the live GitHub REST API on **2026-10-15**; unverified claims are flagged in the final section.

This page is the supporting doc for the spec derived from it. It records *what other projects do and what WM could learn* — not decisions. Design choices live in the spec.

## Already established in this wiki (do not re-research)

| Tool | Where | Verdict already recorded |
|---|---|---|
| Knowns (Go) | @wiki/reference/README, @wiki/concepts/memory-system | Closest peer; parity reached; leapfrog roadmap |
| Graphify (Python) | @wiki/reference/graphify-architecture, @wiki/reference/graphify-adoption-assessment | Adopt edge taxonomy/resolution/cycles; reject clustering + LLM passes |
| OpenSpec | @wiki/decisions/learning-knowns-memory-layer-not-a-spec-system | Complementary spec lifecycle, not a competitor |
| Obsidian / Roam | @wiki/specs/obsidian-graph-view | Graph-view UX benchmark only |
| Reasonix | @wiki/decisions/wm-reasonix-separation | Separate concern; shim only |
| Small "System One" models | @wiki/concepts/system-one-feasibility | No drop-in small Rust model found |

Standing warning: @wiki/concepts/the-fantasy-benchmark-compete-against-expectations-not-reality — first-pass competitor audits idealize rivals. Treat the peer table below as a **design target**, not truth.

## Peer landscape

### Closest structural peers

| Project | Shape | Verified signal | What it does that WM does not |
|---|---|---|---|
| knowns | Go, `.knowns/` in-repo, MCP + ONNX search + LSP code intel + embedded web UI | (unverified stars) | Embedded UI, worktree-isolated agent workspaces, locked spec decisions gated on verified evidence |
| basic-memory | Markdown source-of-truth + SQLite index, MCP-native, Obsidian-compatible | 4,210★, AGPL-3.0, pushed 2026-10-14 | `doctor` file↔DB consistency check, `reindex`, cross-encoder rerank, MCP tool behavior annotations |
| codegraph | `.codegraph/codegraph.db` SQLite+FTS5, Rust tree-sitter, file-watcher auto-sync | (unverified stars) | One default MCP tool; ⚠️ staleness banner inside tool results |
| codebase-memory-mcp / CodeGraphContext | Single static binary + embedded local graph DB | (unverified) | Sub-millisecond structural queries, zero-dependency promise |

### Memory-layer peers

| Project | Storage / index | Retrieval | Signal |
|---|---|---|---|
| beads | Dolt (version-controlled SQL) dependency graph | graph/dependency queries, `bd ready` | 27,886★, Go, MIT, active |
| Hindsight | PostgreSQL + pgvector, "banks" isolation | semantic + BM25 + graph + temporal, merged by RRF + cross-encoder | 49,769★, MIT, active |
| graphiti / Zep | Neo4j/FalkorDB/Neptune; temporal context graph | semantic + BM25 + graph distance rerank | bi-temporal validity windows; invalidate, never delete |
| mem0 | vector-first + entity linking | semantic + BM25 + entity fused | ADD-only extraction; no destructive update |
| cognee | graph + vector (+relational) | graph/vector/code routing | keyless local mode (no API key) |
| mcp-memory-service | sqlite-vec default | ONNX local embeddings, typed causal edges | consolidation + insight cards |
| official `server-memory` | single JSONL file | substring/keyword only | cross-process file lock |
| Cline "Memory Bank" | plain markdown files | none (agent reads files) | manual "update memory bank" friction |

### SDD / workflow peers

| Project | Model | Notable |
|---|---|---|
| spec-kit | constitution → specify → plan → tasks → implement → converge | Skills, not terminal commands; discussion proposes agent may only implement `approved` entities |
| OpenSpec | explore → propose → apply → verify → archive | **Delta specs** — describe the change, not the whole system |
| BMAD-METHOD | guided multi-agent collaboration | Backed off scripted fan-outs toward single outcome-driven SKILL.md |
| claude-task-master | PRD → parse → tasks → expand → implement | Explicit tool tiers ≈ 5k/10k/21k tokens; complexity scores 1–10 |
| Kiro | requirements → design → tasks | EARS notation; requirements-first or design-first |
| Aider repo-map | derived context map | tree-sitter symbol graph + personalized PageRank, token-budgeted |
| Anthropic Agent Skills / AGENTS.md | SKILL.md + AGENTS.md conventions | 3-level progressive disclosure; SKILL.md < 500 lines; 40+ clients |
| Open Knowledge Format (OKF) | markdown + YAML frontmatter, one concept per file | only required field `type`; `index.md` progressive disclosure |

## Actionable lessons (ranked)

1. **Fan one query out to BM25 + vector + graph, merge with RRF, optionally rerank locally.** Every serious peer does parallel retrievers + fusion (Hindsight, mem0, graphiti, claude-context). WM exposes the three modes separately. *Evidence strength: high (multiple independent peers).*
2. **Markdown stays canonical; the graph/index must be provably rebuildable.** basic-memory: files are source of truth, plus `doctor` + `reindex`. Counter-example: Logseq moved the canonical store into SQLite and ships data-loss warnings. *High.*
3. **Non-destructive supersession, not only recency decay.** graphiti invalidates facts via validity windows; mem0 extraction is ADD-only; Hindsight refines rather than overwrites. WM's FSRS-style recency has no supersede semantics. *High.*
4. **Annotate and budget the MCP tool surface.** basic-memory annotates tools (readOnly/destructive/idempotent hints); task-master publishes explicit token tiers; codegraph ships one default tool. WM advertises ~51. *High.*
5. **Surface index staleness inside tool results.** codegraph prepends a ⚠️ dirty-file banner during its debounce window. WM's recurring pain is answering from a stale code index. *High.*
6. **Add a consolidation → briefing pipeline.** Hindsight "mental models" are standing answers with no LLM call; beads `bd prime`; Letta sleep-time. WM has no bounded session-boot context bundle. *Medium-high.*
7. **Carry provenance/evidence on memory and edges, re-checkable.** Hindsight stores evidence quotes + proof counts; graphiti traces facts to episodes. WM's `wm_verify` is a natural enforcement point. *Medium-high.*
8. **Keep the default path LLM-free and fully local — it is the moat.** Peers that mandate a cloud key (mem0, Graphiti/Zep) or an external DB carry that cost; `memento-mcp` was abandoned with a live Neo4j + OpenAI dependency. *High.*
9. **Expose the spec/task tree as MCP resources *and* tools, with a status lifecycle.** spec-kit + knowns lock approved decisions and gate implementation on approval. WM has statuses but no explicit `approved → implement` gate. *Medium (partly from a discussion thread, not a shipped implementation).*
10. **Delta specs and standards interop.** OpenSpec delta specs; import/export OKF bundles, emit SKILL.md-conformant skills so WM stays a superset users can leave. *Medium; OKF evidence is weak — see flags.*

## Anti-patterns to avoid

- Star-count vanity: umbrella-repo stars overstate MCP-server quality.
- Mandatory cloud LLM key or external graph/vector DB as a prerequisite.
- Destructive overwrite of facts ("latest wins, delete the rest").
- Tool-surface bloat — every turn pays the schema tax.
- Demo-grade paths presented as the headline capability.
- Renamed/retired sub-projects as load-bearing dependencies (archived MCP wrappers, deprecated Kuzu).
- Unsafe-by-default network surfaces (an MCP endpoint that is unauthenticated by default).
- Silent schema drift — peers that added a schema guard *refuse* to run a stale binary rather than emit cryptic errors.

## Confidence and unverified claims

- Verified live on 2026-10-15: `basicmachines-co/basic-memory` (4,210★ AGPL-3.0), `gastownhall\/beads` (27,886★ Go MIT; `steveyegge/beads` now redirects there), `vectorize-io/hindsight` (49,769★ MIT Python).
- Weak source: `galkleinman/okf-toolkit` — 2★, last pushed 2026-08-11. Lessons resting on it are ideas, not ecosystem consensus. The OKF format itself (Google Cloud) was not independently re-verified.
- Self-reported by vendors or community tables (not reproduced): mem0 LoCoMo/LongMemEval scores, cognee BEAM, star counts in third-party comparison tables.
- Unverified: star counts for knowns, codegraph, spec-kit, BMAD, serena; `CaviraOSS/OpenMemory` identity.

## Related

- @wiki/reference/graphify-adoption-assessment — the prior adoption assessment this complements
- @wiki/concepts/the-fantasy-benchmark-compete-against-expectations-not-reality — why peer audits must stay honest
- @wiki/concepts/memory-system — WM's current memory model
- @wiki/decisions/learning-knowns-memory-layer-not-a-spec-system — incumbent positioning