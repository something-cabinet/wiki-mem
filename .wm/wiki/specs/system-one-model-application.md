---
title: System One Decision Model — Feasibility for Wiki-Mem
type: spec
id: "wiki:specs:system-one-model-application"
status: approved
tags: [system-one, decision-model, feasibility, onnx, laya, research, small-models, approved]
---

## Overview

Research spike to determine whether a local, free **System One decision model** — given a *state* it returns typed, calibrated answers (`choice` / `score` / `noul`) in a single forward pass, with no generated text — can replace or augment heuristics across five Wiki-Mem workflows, on the **typical Wiki-Mem developer machine: ~8 GB RAM, no GPU**. The deliverable is a per-workflow verdict. **No code changes are made by this spike.**

## Locked Decisions

Decisions extracted during exploration:

- **D1**: Goal is *feasibility/application research* of a System One decision model to WM — not a keep/remove-ONNX decision.
- **D2**: Analysis covers all five dimensions — build/binary cost, runtime cost, search quality, platform/distribution risk, maintenance/complexity.
- **D3**: Evaluate all five WM workflows — search relevance & routing, page & task classification, memory & rule decisions, MCP tool routing & review gates, and graph edge typing.
- **D4**: Research spike only — no code changes; output is a written feasibility spec + recommendation.
- **D5**: **Local, free, open-source only.** Hosted decision APIs (e.g. TypeSafe Jev) are explicitly out of scope.
- **D6**: Fine-tuning a model on WM's own data **is in scope** and is considered feasible.
- **D7**: **Target hardware is a developer machine with ~8 GB RAM and no GPU.** The old ONNX embedder (~198 MB) is a *reference for the small-footprint class*, **not a hard cap**. The real constraint is that a model must run comfortably in RAM on an 8 GB, CPU-only machine while the OS, editor, and browser are also open — a candidate that cannot is ineligible for the default path.
- **D8**: Report a **per-workflow verdict for all five**, each with feasibility, a shortlisted small local model, a rough eval plan, and a GO / NO-GO / NEEDS-FINETUNE call.

## Requirements

### Functional Requirements

- **FR-1**: Explain the System One decision-model paradigm (`choice`/`score`/`noul`, single forward pass, calibrated confidence, no generated text) and contrast it with WM's current BM25 + embedding / RRF pipeline.
- **FR-2**: Inventory the local, free, open-source, Rust-runnable System One ecosystem (crates + model weights) and identify the subset that runs in-process on a CPU-only, ~8 GB RAM machine.
- **FR-3**: For each of the five workflows, describe the concrete decision to be made — the *state*, the typed *questions*, and the *answer space*.
- **FR-4**: For each workflow, shortlist at least one candidate model that fits D7, documenting params, on-disk size, peak RAM, license, runtime crate, and any known benchmark/calibration figures.
- **FR-5**: For each workflow, propose an evaluation design — dataset/labels, baseline (current heuristic or current behaviour), metric (accuracy / ECE / latency), and a pass threshold.
- **FR-6**: Assess fine-tuning feasibility — data format, minimum labelled examples, local tooling, export format, and the swap-in path that requires no code change.
- **FR-7**: Assign each workflow a verdict — **GO / NO-GO / NEEDS-FINETUNE** — with rationale.
- **FR-8**: Identify the highest-value single workflow to prototype first, if any workflow is GO or NEEDS-FINETUNE.

### Non-Functional Requirements

- **NFR-1**: Every candidate must run on CPU without a GPU and with no network at runtime (fully in-process/offline), on a machine with ~8 GB RAM while normal dev tools are also running.
- **NFR-2**: Every candidate's footprint must be characterized by **both on-disk size and peak resident RAM** on a CPU-only machine, and judged against the ~8 GB RAM target. The ~198 MB embedder is the reference class for "small", not a pass/fail threshold.
- **NFR-3**: No third-party API dependency for core functionality (per `core/ARCHITECTURE` non-negotiable).
- **NFR-4**: All model and crate licenses must be permissive (Apache-2.0 / MIT) and free for commercial use.
- **NFR-5**: Findings must cite exact source URLs; unverifiable claims must be marked "unknown".
- **NFR-6**: This spike is advisory only — it must not change code, dependencies, or config.

## Acceptance Criteria

- [ ] AC-1: A written report exists covering all five workflows, each with a GO / NO-GO / NEEDS-FINETUNE verdict.
- [ ] AC-2: The report explains the System One paradigm and contrasts it with the current BM25/hybrid search pipeline.
- [ ] AC-3: At least one CPU-only, permissively-licensed, free candidate model is named per workflow, with params, on-disk size, peak RAM, and license.
- [ ] AC-4: Each candidate is assessed for whether it runs comfortably on an ~8 GB, CPU-only machine alongside normal dev tools (the ~198 MB embedder is the reference class, not a hard cap).
- [ ] AC-5: Each workflow has a proposed evaluation design with metric and pass threshold.
- [ ] AC-6: Fine-tuning feasibility is assessed with data format, minimum examples, local tooling, and export/swap path.
- [ ] AC-7: The report recommends one workflow to prototype first (or states none, if all verdicts are NO-GO).
- [ ] AC-8: All claims are cited with exact URLs; unknowns are explicitly marked.
- [ ] AC-9: No code, dependency, or config change is made by this spike.

## Scenarios

### Scenario 1: A workflow is a GO
**Given** a WM workflow
**When** the report finds a candidate that runs comfortably on an 8 GB CPU-only machine and shows credible benchmark improvement over the baseline
**Then** it records a GO verdict with the candidate, eval design, and expected footprint.

### Scenario 2: A workflow needs fine-tuning
**Given** a workflow where no off-the-shelf model is adequate
**When** fine-tuning on WM's own labels is feasible
**Then** the verdict is NEEDS-FINETUNE, with the dataset spec and local training path.

### Scenario 3: Footprint too large for the target machine
**Given** a strong model whose peak RAM would not fit an 8 GB CPU-only machine alongside normal dev tools (e.g. Laya at ~421M → ~0.85 GB f16)
**When** it is evaluated
**Then** it is marked ineligible for the default path, and may be noted only as an optional high-spec/quantized tier.

### Scenario 4: No viable candidate
**Given** a workflow whose decision cannot be expressed as `choice`/`score`/`noul`, or for which no model fits D7
**When** it is evaluated
**Then** the verdict is NO-GO with the reason recorded.

## Technical Notes

- **Target machine**: ~8 GB RAM, CPU-only, no GPU — the model must coexist with OS + editor + browser. Interpret "small" as "comfortably resident", not a fixed disk number.
- **Runtime paths**: safetensors + candle (crate `laya`, repo `aovestdipaperino/laya-rust`, MSRV 1.75, Apache-2.0, sync `Agent::system_one(&state, &Vec<(String, Question)>)`) vs ONNX + `ort` (crate `jigor`, daemon `ollaya`).
  - Name collision warning: the crates.io package `laya-rust` is a **different** project; the repo `aovestdipaperino/laya-rust` publishes as crates.io `laya`.
- **Fine-tune tooling**: System One Studio / Laya Studio (Apache-2.0, local, `layastudio`), MLX on Apple Silicon; export emits `model.safetensors` (candle-compatible) and `model.onnx` (opset 18).
- **Calibration** is per `(question type, option count)` and must be refit on WM data; base checkpoints are weak zero-shot (Laya root ~chance on typed-decisions).
- **Footprint arithmetic**: `bytes ≈ params × 2` (f16) or `× 4` (f32); int8 ≈ `params × 1`. Approximate candidate sizes to verify:

  | Model | Params | f16 | f32 | int8 | 8 GB fit? |
  |---|---|---|---|---|---|
  | GLiNER2.5-small | 74M | ~148 MB | ~296 MB | ~74 MB | likely |
  | Julia-1 | 141M | ~282 MB | ~564 MB | ~141 MB | likely |
  | Verdict | 151M | ~302 MB | ~604 MB | ~151 MB | likely |
  | Decision-1.0-Kai | 308M | ~616 MB | ~1.23 GB | ~308 MB | borderline |
  | LFM2.5-350M | 350M | ~700 MB | ~1.40 GB | ~350 MB | borderline |
  | Laya | 421M | ~842 MB | ~1.68 GB | ~421 MB | over (default path) |

- **Integration surface if a GO occurs later**: `wm-embed`'s ONNX pipeline and the `onnx` cargo feature (currently default in `wm-cli`/`wm-core`/`wm-server`).
- **Baseline to beat**: WM's current hybrid search (BM25 + RRF + post-rerank) and existing heuristics; the ~198 MB ONNX embedder is the reference footprint class.

## Open Questions

- [ ] Which workflow is the intended first target (affects the depth of the eval design)?
- [ ] What is the acceptable peak-RAM ceiling on an 8 GB machine (e.g. ≤300 MB, ≤500 MB)?
- [ ] Should a decision model *replace* an existing heuristic or run as an additive confidence signal?
- [ ] Where would labelled WM training/eval data come from (existing tasks/pages/memory vs synthetic)?