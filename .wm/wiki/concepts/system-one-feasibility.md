---
title: System One Decision Models — Feasibility Report (Wiki-Mem)
type: concept
id: "wiki:concepts:system-one-feasibility"
status: draft
tags: [system-one, decision-model, feasibility, report, onnx, laya, small-models, reviewed]
---

schema_version: 1
state: |-
  ## Summary

  Research spike verdict: a **local, free System One decision model can plausibly be applied to Wiki-Mem, but there is no small, drop-in Rust System One model today.**

  - The only Rust-native System One crates serve models **≥322M** (Laya) whose CPU footprint (~0.85 GB disk, ~2.4 GB peak) is **borderline on the target ~8 GB / no-GPU machine**.
  - Sub-200M decision models exist and expose full `choice`/`score`/`noul` — **Verdict-small (118M)**, **Julia-1 (144M)**, **Verdict (151M)** — but ship **ONNX/PyTorch, not a Rust System One crate**. They could run via a **thin `ort` wrapper**, and WM already depends on `ort` through `wm-embed`.
  - **Recommended first prototype: graph edge typing**, chosen for its bounded target and existing edge corpus. It must be run as a **three-way head-to-head**: (a) `gliner-rs` (74M, Rust-native, choice-only), (b) a frozen existing `ort` embedder + trained linear head (cost floor), and (c) a custom-`ort` sub-200M decision model. See **Recommendation**.

  > Evidence note: this report was revised after independent oracle review (two passes). Baseline commit at time of writing: **`60e43f7`** (the working tree also contained unrelated TUI-removal changes; no code was changed by this spike). Citations are in **Sources**.

  ## Method and constraints

  - **Target**: ~8 GB RAM, CPU-only, no GPU; fully offline/in-process.
  - **Hard constraints**: free; permissive license (Apache-2.0/MIT); **no Python/Node backend services** (per `core/ARCHITECTURE`); no third-party API dependency for core.
  - **D2 dimensions assessed**: build/binary cost, runtime cost, search quality, platform/distribution risk, maintenance/complexity.
  - **Sources**: librarian research on the System One / Laya / Jev Decision Index ecosystem (lib-1, lib-2); codebase recon of the five WM workflows (exp-4); independent oracle review (ora-1, ora-2). Aliases are mapped in **Sources**.

  ## The paradigm vs WM's pipeline

  A System One model takes a **state** plus **typed questions** (`choice` — pick one of N; `score` — ordered level; `noul` — P(condition true)) and returns **calibrated probabilities in a single forward pass**, with no generated text. WM today is deterministic heuristics: frontmatter string parsing, enum lookup, BM25 + RRF + post-rerank, FSRS salience. A decision model would **augment discrete decision points**, not replace retrieval.

  ## Ecosystem and runtime reality (the crux)

  | Rust runtime | Format | Models it actually loads | Drop-in for a small model? |
  |---|---|---|---|
  | `laya` crate (candle) | safetensors | Laya only (322M/421M) | ✗ |
  | `gliner-rs` (candle) | safetensors | GLiNER2 boundary models (choice-ish only) | ~ choice-only |
  | `jigor` (`ort` ONNX) | ONNX | von (395M), laya (322M/421M), jeff (0.8B), remote | ✗ |
  | `ollaya` (ONNX Runtime) | ONNX | 15 families, smallest 322M (laya) / 395M (von) | ✗ |
  | **`ort` (already in `wm-embed`)** | ONNX | any ONNX graph (custom wrapper needed) | ✔ with custom code |

  **No sub-200M model has a prebuilt Rust System One crate.** The realistic small path is a thin custom `ort` wrapper around a sub-200M ONNX decision model — `ort 2.0.0-rc.12` + `tokenizers` are already present behind the default-on `onnx` feature (`packages/wm-embed/Cargo.toml`).

  **Crate-name collision (from the spec):** the crates.io package **`laya-rust`** is an unrelated project (CUDA/GGUF engine); the repo `aovestdipaperino/laya-rust` publishes as crates.io **`laya`**. All "laya crate" references here mean the latter.

  ## Candidate shortlist (verified)

  | Model | Params | On-disk | License | Runtime | primitives | Reported acc/ECE | Peak RAM | 8 GB fit |
  |---|--:|---|---|---|:--:|---|---|---|
  | GLiNER2.5-small | 74M | 281.9 MiB sf (FP32) | Apache-2.0 | `gliner-rs` (candle) | choice-ish only | Index skill 3.82 / raw 25.99, ECE 0.1608, 13.6 ms | unknown | comfortable |
  | GLiNER2.5-base | 194M | 738.5 MiB sf | Apache-2.0 | `gliner-rs` | choice-ish only | (same family) | unknown | comfortable |
  | Verdict-small | 118M | 112.6 MiB ONNX int8; 224.4 MiB ONNX fp16 | Apache-2.0 | custom `ort` | choice/score/noul | zero-shot Banking77 0.556, SST-5 0.403; Index skill 1.87, ECE 0.154, 11.4 ms | unknown | comfortable |
  | Julia-1 | 144M | 550.5 MiB sf; 552.9 MiB ONNX | Apache-2.0 | custom `ort` | **full** | typed-decisions 73.15% (CUDA BF16); CPU FP32 choice 71.0% / score 67.75% / noul 80.5% | **393.1 MB RSS documented** | comfortable |
  | Verdict (151M) | 151M | 577.5 MiB sf; 289.7 MiB ONNX fp16 | Apache-2.0 | custom `ort` | **full** | held-out top-1 95.0%, Brier 0.0785, ECE 3.35%; Index broad skill 1.87, ECE 0.154, p50 35.58 ms | unknown | est. OK (unmeasured) |
  | Lumma-Fev-0.1B | 154M | 295.6 MiB sf (bf16) | Apache-2.0 | none (Python) | full | typed-decisions 0.49, AG News 0.89, avg 0.63 | unknown | comfortable |
  | Decision-1.0-Kai (Kai-0.6B) | ~0.57B | 2181.7 MiB sf total | Apache-2.0 (mixed notices) | none (vLLM-SR) | full | Index skill 6.52, ECE 0.1851, 30.4 ms | unknown (weights 2.13 GiB on disk) | ⚠️ flagged |
  | LFM2.5-350M | 354M | 676.1 MiB sf | **other (Liquid; not permissive)** | none (generative) | n/a (chat model) | Index skill 1.38, ECE 0.568 | unknown | ✗ license + not a decision model |
  | **Laya (multilingual)** | 322M | ~684 MiB | Apache-2.0 | **`laya` crate (candle)** | **full** | raw ECE 0.466 → 0.081 post-temp; Index skill 6.04, ECE 0.1402, 5.8 ms | ~2.4 GB peak fp32 CPU (crate docs) | borderline |
  | **Laya (English)** | 421M | ~854 MiB | Apache-2.0 | `laya` crate | **full** | (as above) | ~2.4 GB peak fp32 CPU | borderline |

  Peak RAM is marked **unknown** where the source does not document it; it is **not** inferred from disk size (per NFR-2). Only Julia-1's 393.1 MB CPU RSS and Laya's ~2.4 GB are documented.

  ## Considered and rejected

  - **bekko-system-one (17M / 68M / 400M)** — browser/WASM demos; full choice/score/noul contract and license were **not verifiable**; excluded pending verification.
  - **System One Mini (69M)** — fixed 5-head schema; card states it is not a general reproduction.
  - **GLiNER2.5-Decide (486M)** — over the small footprint; single-purpose classification.
  - **`sys1` / `laya-candle`** — alternative candle servers; not wired to any shortlisted small model.
  - **`tract-onnx`** — pure-Rust ONNX loader; a fallback if the `ort` RC dependency is undesirable.
  - **WASM delivery** — not viable for the candle/`tokenizers` path (`onig_sys` cannot build for wasm32); only models with prebuilt web runtimes have a browser route.

  ## Per-workflow assessment

  ### 1. Search relevance & routing
  - **Framing**: `choice` mode (keyword/semantic/hybrid/auto), `score` relevance, `noul` is-relevant.
  - **Baseline**: `apps/wm-core/src/search/query.rs:100-417` (RRF fusion, tie-break tiers); `packages/wm-search/src/services/bm25_index_service.rs:199-331,360-444` (BM25 + boosts); `packages/wm-embed/src/models/search_mode_model.rs:36-45` (`auto_detect`, a 2-branch string heuristic). Eval harness `apps/wm-core/tests/golden_eval.rs` is **keyword-only, synthetic, and `#[ignore]`d** (`RECALL_FLOOR = 0.85` is a fixture floor, not a production measurement).
  - **Fit**: mode routing is a trivial choice already handled adequately; relevance is already strong and a small zero-shot model will not beat BM25+RRF+rerank without training.
  - **Eval design**: extend to **real logged queries with the `mode` parameter**; report routing accuracy separately from macro recall@5 / MRR; pass = no regression below the measured baseline and +0.05 recall@5 on the hybrid path.
  - **Verdict: NO-GO** (keep heuristics; revisit only if a real-query eval shows a gap).

  ### 2. Page & task classification
  - **Framing**: `choice` type (11-way), `choice` status (per-type), `score` priority (4-level), multi-label tags.
  - **Baseline**: `apps/wm-core/src/parser/mod.rs:90-141` (string lookup) + `:286-319` fallbacks (unknown type → Concept silently; unknown status → Draft **warns**, absent status silently defaults); enums in `packages/wm-engine/src/models/page_type_model.rs:8-163`. Dir-name inference only. Corpus is skewed (task 286, note 1).
  - **Fit**: bounded choice — GLiNER2.5-small / Julia-1 / Verdict-small. Because types are authored explicitly, the honest use is **validation / mismatch detection**, not silent assignment (dir-name already ≈ the authored type, so there is little headroom in-distribution).
  - **Eval design**: build a **synthetic corruption corpus** (valid pages with perturbed type/status/priority); report precision/recall of mismatch detection and macro-F1 **excluding path/dir from the state** to avoid leakage; define rare-class handling.
  - **Verdict: GO (low-risk, bounded)**, scoped to validation/suggestion.

  ### 3. Memory & rule decisions
  - **Framing**: `noul` store/promote, `choice` layer, `choice` dedup action, `noul` duplicate/conflict.
  - **Baseline**: `apps/wm-core/src/mcp/tools/memory.rs:106-128` (FSRS eviction), `:216-432` (promote = file copy); dedup/merge/promotion are **agent prose heuristics** in `wm-extract/SKILL.md:283-334,395-470`. No automated conflict detection; `search/memory.rs` is a stub.
  - **Fit**: `noul`/`choice` — Julia-1 / Verdict. **No decisions corpus exists.**
  - **Eval design**: define an annotation protocol — who labels, sample size (≥200 decisions), inter-annotator agreement ≥0.8; target precision ≥ 0.90 / recall ≥ 0.70 on dedup + promotion.
  - **Verdict: NEEDS-FINETUNE** (highest human value, zero labelled corpus).

  ### 4. MCP tool routing & review gates
  - **Framing**: `choice` tool, `choice` review verdict, `score` severity (P0–P3), `noul` block?.
  - **Baseline**: `apps/wm-core/src/mcp/tools/project.rs:64-186` (`wm_help` substring + agent reasoning); `wm-review/SKILL.md:85-145` prose rubric (P1 blocks commit; verdict vocabulary is Approve / Changes-requested / Blocked); `apps/wm-core/src/mcp/tools/validate.rs:43-327`.
  - **Fit**: the LLM agent **already performs this**; a small local model risks misrouting tools and mis-grading review severity — safety-critical. **No logged decision traces.**
  - **Eval design**: tool-choice accuracy and verdict agreement vs logged agent decisions — **blocked: no corpus**.
  - **Verdict: NO-GO** (out of scope for a local small model; safety-critical).

  ### 5. Graph edge typing
  - **Framing**: `choice` edge type (12 variants declared + Custom), `choice` provenance, `choice` target resolution.
  - **Baseline**: `apps/wm-core/src/parser/mod.rs:337-360` (`relates_to` parse; default `relates_to` for body links), `packages/wm-engine/src/models/edge_type_model.rs:69-142`, `apps/wm-core/src/graph/mod.rs:154-192` (Ambiguous resolution), `packages/wm-code-intel/src/services/graph_resolver.rs:158-515` (code edges).
  - **Corpus reality (strict frontmatter parse, de-duplicated)**: **280 typed edges** — `references` 196 (70%), `implements` 64 (23%), `relates_to` 7, `superseded_by` 6, `example_of` 4, `supersedes` 2, `part_of` 1; **zero** `extends` / `depends_on` appear in frontmatter, and **zero** wikilink double-bracket syntax exists in the tree. The remaining classes have **0–7** examples. (A raw text grep returns ~326 because of YAML examples in specs/concepts and the duplicated frontmatter blocks on core pages — do not reuse that number.)
  - **Fit / scope**: macro-F1 ≥ 0.80 across all 12 classes is **not supportable**. Scope the prototype to classes with **≥30 examples** (`references`, `implements`) or collapse rare classes into an `other` bucket; evaluate the `code_edges` table **separately** as cross-domain transfer, not as augmentation.
  - **Eval design**: held-out split of the ≥30-example classes; metric = macro-F1 over the scoped classes + per-class P/R for rare classes; **per-model** latency budget (see Recommendation), not a single threshold.
  - **Verdict: GO**, scoped as above — **first prototype**.

  ## Fine-tuning feasibility

  - **Data format**: JSONL rows `{"state": <text|json>, "answers": {"<qid>": <label|prob>}}`; soft labels allowed; keep the questions object identical after training.
  - **Volume**: ~30 examples per label to start, 100+ to be solid, 200+ test decisions.
  - **Calibration**: refit temperature per `(question type, option count)` on held-out data; base checkpoints are weak zero-shot.
  - **Trainer/export per candidate**:

    | Candidate | Trainer | Export |
    |---|---|---|
    | Laya | System One Studio (MLX, Apple Silicon) | safetensors + ONNX opset 18 |
    | Verdict / Verdict-small | project `[train]` Python | ONNX published |
    | Julia-1 | PyTorch / `julia-mlx` | safetensors + ONNX |
    | GLiNER2.5-small | Fastino `gliner2` trainer | safetensors (candle) |

  - **Swap-in contract (FR-6)**: a **same-architecture** fine-tune swaps the ONNX/safetensors artifact + config with no code change; a **cross-family** swap needs a new backend module. The no-code-change guarantee holds only within a family.
  - **Dev-time training, Rust inference**: training is an offline, pre-release step that emits artifacts; it introduces no Python *service*, so it does not violate the architecture rule.

  ## D2 dimensions

  - **Build / binary cost**: `ort 2.0.0-rc.12` + `tokenizers` are already compiled behind the `onnx` feature; a decision backend adds a module but no new heavy dependency. `tract-onnx` is a pure-Rust alternative if the `ort` RC is a concern.
  - **Runtime cost**: documented peaks are Julia-1 393.1 MB and Laya ~2.4 GB; all others **unmeasured** — measure before committing a default path.
  - **Platform / distribution risk**: model weights are **not** shipped in-repo; reuse the existing ONNX model-download path (with integrity verification), and keep default-on inference behind verification (see `WM-005`).
  - **Maintenance / complexity**: one backend module per model family; same-family fine-tunes are artifact swaps. Keep the decision model behind a feature flag so users who never enable it are unaffected.

  ## Recommendation

  **First prototype — graph edge typing**, scoped to classes with ≥30 examples, as a **three-way head-to-head**:

  1. **`gliner-rs` + GLiNER2.5-small (74M)** — Rust-native (candle), no new inference code, choice-only (no `score`/`noul`), but edge typing is a pure choice task.
  2. **Frozen existing `ort` embedder + trained linear head** — the true cost floor; uses embeddings already computed by `wm-embed`.
  3. **Custom `ort` + sub-200M decision model** (Verdict-small int8 112 MiB as light default; Julia-1 144M if a full three-primitive model is wanted) — the challenger.

  Pursue (3) only if it beats (1) and (2) on edge typing. Add a **post-quantization ECE gate**: fall back to fp16 if int8 ECE regresses beyond the agreed threshold.

  - **Do not adopt Laya as the default** (~2.4 GB peak is too heavy for 8 GB); optional high-spec tier only.
  - **Do not treat GLiNER2.5 as a System One model** — it is choice-only.
  - **Second candidate workflow** — memory & rule decisions, gated on the label-collection step.
  - **Skip** search relevance/routing and MCP tool routing/review gates.

  ## Open Questions

  - [ ] Peak RAM of the sub-200M ONNX models under `ort` on an 8 GB CPU machine is unmeasured — measure before choosing a default.
  - [ ] Are the 280 edges (70% `references`) sufficient to train/evaluate edge typing, or is more data needed for the rare classes?
  - [ ] Which model to standardise on (Verdict-small vs Julia-1 vs `gliner-rs`) after the head-to-head.
  - [ ] Is an offline, dev-time Python training step acceptable, given inference stays Rust?
  - [ ] Is the `ort` 2.0.0-rc dependency acceptable, or is `tract` preferred?

  ## Sources

  **Research/review aliases cited above:** lib-1 = `laya-rust` repo research; lib-2 = Jev Decision Index + fine-tuning research; exp-4 = WM five-workflow codebase recon; ora-1/ora-2 = independent feasibility-report reviews.

  - crates.io: `laya` https://crates.io/crates/laya · `gliner-rs` https://crates.io/crates/gliner-rs · `jigor` https://crates.io/crates/jigor · `gline-rs` https://crates.io/crates/gline-rs · `tract-onnx` https://crates.io/crates/tract-onnx
  - repos: `gliner-rs` https://github.com/apiplant/gliner-rs · Fastino GLiNER2 https://github.com/fastino-ai/GLiNER2 · Laya Studio https://github.com/biplovgautam/LayaStudio · Julia MLX https://github.com/zm2231/julia-mlx · Verdict (151M) https://github.com/Heman10x-NGU/Verdict-open-jev · Verdict-small https://github.com/Manavarya09/verdict · `aovestdipaperino/laya-rust` https://github.com/aovestdipaperino/laya-rust
  - Hugging Face models: Laya https://huggingface.co/convaiinnovations/laya · Julia-1 https://huggingface.co/SupersonicLabs/Julia-1 · Julia-1 ONNX https://huggingface.co/SupersonicLabs/Julia-1-ONNX · Verdict 151M https://huggingface.co/heman10x/rlcd-modernbert-151m · Verdict-small https://huggingface.co/Manav2op/verdict-small · GLiNER2.5-small https://huggingface.co/fastino/gliner2.5-small-v1 · GLiNER2.5-base https://huggingface.co/fastino/gliner2.5-base-v1 · Decision-1.0-Kai-0.6B https://huggingface.co/llm-semantic-router/Decision-1.0-Kai · LFM2.5-350M https://huggingface.co/notnotsamuel/LFM2.5-350M-RLCD · Lumma-Fev-0.1B https://huggingface.co/FrontiersMind/Lumma-fev-0.1b · System One Mini https://huggingface.co/DavidHatley/system-one-mini · laya-onnx https://huggingface.co/Mattepiu/laya-onnx
  - Spaces / platforms: Jev Decision Index https://huggingface.co/spaces/multimodalart/jev-decision-index · ollaya https://ollaya.dev/ and https://ollaya.dev/library · systemonemodels.tech https://systemonemodels.tech/ · systemonemodels.org https://systemonemodels.org/ · Layastudio site https://layastudio.biplovgautam.com.np/ · Julia-1 site https://supersoniclabs.ia.br/julia-1/ · NoulXP https://github.com/systemonemodels/noulxp
  - Unverified / unknown: `bekko-system-one` contract + license; peak RAM for all candidates except Julia-1 and Laya; whether Studio-exported ONNX is a drop-in for `jigor`/`ollaya`; the `sevenreasons/von-onnx-fp16` weights (gated); Laya per-variant sizes (HF repo total is 2221.2 MiB across variants).
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
