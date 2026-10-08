---
title: 'Eval: gliner2.5-small-v1 typed-decision runtime'
type: concept
id: "wiki:concepts:gliner-decision-runtime-eval"
status: draft
record_format: prose
relates_to:
  - {type: relates_to, target: wiki:specs:eval-gliner-typed-decisions}
---

schema_version: 1
state: |-
  ## Overview

  End-to-end measurement of the `gliner2.5-small-v1` typed-decision runtime on 30 real WM pages (114 labeled canonical questions), per `@wiki/specs/eval-gliner-typed-decisions`. Harness: `apps/wm-core/tests/decision_eval.rs` (+ `decision_eval_mock.rs`); fixture: `apps/wm-core/tests/fixtures/decision_eval.jsonl`; raw results: `apps/wm-core/tests/fixtures/decision_eval_results.json`.

  Verdict: **gliner2.5-small-v1 is NOT useful for WM typed decisions as shipped.** Overall accuracy (0.404) is below a trivial per-question majority baseline (0.623); p50 latency is 27 s/row on CPU; calibration is mediocre (noul ECE 0.223). It is also close to chance on exact-label choice.

  ## Results

  | Metric | gliner2.5-small-v1 | Majority baseline |
  |---|---|---|
  | Overall accuracy (n=114) | 0.404 (46/114) | **0.623** |
  | `choice` accuracy (n=42) | 0.405 (17/42) | — |
  | `choice` macro-F1 | 0.369 | — |
  | `noul` accuracy (n=42) | 0.500 (21/42) | — |
  | `noul` ECE | 0.223 | — |
  | `score` accuracy (n=30) | 0.267 (8/30) | — |
  | `score` ECE | 0.148 | — |
  | Latency p50 | 27.2 s | — |
  | Latency p90 | 127.5 s | — |

  Accuracy by page type: `concept` 0.583, `reference` 0.444, `pattern` 0.375, `decision` 0.333, `howto` 0.292.

  The baseline is inflated by the `decision` fixture being all `outcome: adopted` (all 6 decision pages are accepted ADRs); even so, the model loses to it.

  ## Run recipe

  1. Build: `cargo build -p wm-cli --features decision`.
  2. Download the checksum-pinned model (282 MiB, verified per-file SHA-256): call the `wm_model` tool with `{"action":"download","name":"gliner2.5-small-v1"}` (MCP), or place the verified files under `~/.wm/models/gliner2.5-small-v1/`. Note: the `wm model download` CLI subcommand still routes only the ONNX embedding models, not the decision manifest — use the MCP `wm_model` path.
  3. Mock harness (no model): `cargo test -p wm-core --test decision_eval_mock -- --nocapture`.
  4. Real run (needs the model): `cargo test -p wm-core --features decision --test decision_eval -- --ignored --nocapture` (~25 min CPU).

  ## Method

  - Fixture: 30 real `.wm/wiki` record `state` bodies (6 each of decision/pattern/concept/howto/reference), 114 hand-labeled answers covering `choice` (42), `noul` (42), and `score` (30). `score` labels are flagged `soft` (subjective) but still evaluated so ECE is reported; `reference` `kind`/`surface` on the Graphify/design-patterns pages are left unlabeled (taxonomy mismatch).
  - `DecisionRuntime` runs the real `GlinerBackend`; metrics: accuracy per primitive + overall, per-question macro-F1 for `choice`, 10-bin ECE for `noul`/`score`, and wall-clock p50/p90.
  - Baseline: per (page_type, question_id) majority/constant from the fixture.

  ## Caveats

  - Real pages still carry migration noise (duplicate frontmatter echoed into `state`), which likely depresses accuracy — but that is the data the runtime sees today.
  - `score` ground truth is subjective; treat score numbers as indicative only.
  - n=30/114 is small for stable ECE.
  - Latency is dominated by long states being chunked (384-word windows) on CPU.
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
