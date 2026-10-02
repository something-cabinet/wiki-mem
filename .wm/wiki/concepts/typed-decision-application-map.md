---
title: Typed-Decision Application Map — Where a System One Model Fits in the WM Pipeline
type: concept
id: "wiki:concepts:typed-decision-application-map"
status: draft
tags: [typed-decisions, system-one, pipeline, retrieval, ranking, decision-layer]
relates_to:
  - {type: relates_to, target: wiki:specs:typed-decision-doc-format}
---

schema_version: 1
state: |-
  ## Purpose

  Companion to `@doc/concepts/system-one-feasibility`. Clarifies **where a System One typed-decision model (`choice`/`score`/`noul`) fits in the Wiki-Mem pipeline**, where it does **not** apply, and how it relates to sparse search, embeddings, and ranking. Decision summary: it is a **decision layer**, not a retrieval layer.

  ## It is a decision layer, not a retrieval layer

  | Layer | Question it answers | WM today | System One? |
  |---|---|---|---|
  | **Retrieval (recall)** | "Which items in the corpus match this prompt?" | BM25 sparse + dense embeddings + graph traversal | ✗ |
  | **Ranking** | "In what order?" | RRF fusion + post-RRF rerank | ✗ (optional reranker only) |
  | **Decision** | "Given this state, pick / score / judge?" | Heuristics + agent prose | ✔ |

  Typed decision **sits downstream of retrieval** (on candidate sets) and at **standalone discrete decision points**. It does not index, search, or produce recall.

  ## What it does NOT replace

  - **Sparse search (BM25)** — a decision model has no inverted index and no corpus scan; it cannot retrieve. Recall stays with BM25.
  - **Dense embeddings / semantic matching (synonyms, paraphrase)** — embeddings map meaning to vectors; a decision model has no similarity index and **cannot generate** synonyms or expand a query (no generation; it only picks among given options). Synonyms are an embedding-layer concern.
  - **Retrieval ranking (RRF / post-rerank)** — at most it can act as an optional **cross-encoder reranker** over the top-k (one inference per candidate; precision, not recall), and even that is currently NO-GO.
  - **Current state caveat**: semantic search in this project is **degraded** ("ONNX model not loaded"), so retrieval is effectively sparse-only — thus **no synonym handling today**. The fix is the **embedder**, not a decision model.

  ## Where typed decisions apply in the WM pipeline

  | # | Pipeline stage | Decision to make | Primitives | Current baseline | Verdict |
  |---|---|---|---|---|---|
  | 1 | Ingest / page write | classify + validate `type`/`status`/`priority`/`tags` | choice / score | string lookup + dir-name; silent fallbacks (`apps/wm-core/src/parser/mod.rs:90-141,286-319`) | **GO** (validation / mismatch detection only) |
  | 2 | Graph linking | typed edge between two pages | choice | authored `relates_to`; body links default to `relates_to` (`parser/mod.rs:337-360`) | **GO** — first prototype |
  | 3 | Memory & rules | store? promote? dedup? conflict? | noul / choice | FSRS eviction + file-copy promote; dedup is agent prose (`mcp/tools/memory.rs:106-432`, `wm-extract/SKILL.md`) | was NEEDS-FINETUNE → **NO-GO now** (training excluded) |
  | 4 | Search | mode routing (keyword/semantic/hybrid); optional rerank | choice / noul / score | `auto_detect` heuristic + BM25+RRF+rerank (`search/query.rs:100-417`) | **NO-GO** (strong baseline; no recall benefit) |
  | 5 | MCP / agent | which `wm_*` tool to call | choice | agent reasoning + `wm_help` (`mcp/tools/project.rs:64-186`) | **NO-GO** (safety-critical; no traces) |
  | 6 | Review gates | GO / GO-with-findings / NO-GO; severity | choice / score | prose rubric (`wm-review/SKILL.md`) | **NO-GO** |
  | 7 | Doc format | make every doc a typed-decision record | choice / score / noul | prose bodies | **GO** — `@doc/specs/typed-decision-doc-format` |

  ## Rule of thumb

  Retrieval answers **"what might be related?"**; a typed-decision model answers **"given this, pick / score / judge."** They are complementary layers — a judge, not a search engine.

  ## Implications

  - **Keep** BM25 + RRF + post-rerank as the retrieval/ranking core.
  - **Restore/fix the ONNX embedder** for synonym and semantic recall — that is the real lever for "find related" and synonyms.
  - **Apply typed decision** at: graph **edge typing** (first prototype), page/task **validation**, and the **typed-decision doc format**.
  - **Do not** expect typed decision to improve recall, handle synonyms, or replace ranking.

  ## References

  - @doc/concepts/system-one-feasibility
  - @doc/specs/typed-decision-doc-format
  - @doc/specs/system-one-model-application
questions:
  - id: kind
    type: choice
    instructions: What kind of concept document is this?
    options:
    - concept
    - failure-analysis
    - research-report
    - reference-note
  - id: category
    type: choice
    instructions: Which domain category does this concept belong to?
    options:
    - architecture
    - search-retrieval
    - graph
    - parser-format
    - mcp-tooling
    - cli
    - storage
    - embeddings
    - web-ui
    - process
  - id: maturity
    type: score
    instructions: How mature is the understanding of this concept?
    levels:
    - raw
    - exploratory
    - established
    - stable
  - id: code_referenced
    type: noul
    instructions: This concept references concrete code.
answers: {}
