---
title: gliner Decision-Quality Experiment — Findings & Verdict
type: concept
id: "wiki:concepts:gliner-decision-quality-findings"
status: draft
tags: [gliner-rs, typed-decisions, eval, findings, verdict, calibration]
---

schema_version: 1
state: |-
  ## Summary

  On a fixed **30-page / 114-question** fixture, **no configuration of the local gliner runtime beats the per-question majority baseline** (aggregate **0.623**). Best config: `GLiNER2.5-Decide` + `key-facts` + `label-descriptions` = **0.491** (15/114 questions short of the constant). The local typed-decision runtime is **not viable as-is** for WM decisions.

  ## Setup

  - Fixture: `apps/wm-core/tests/fixtures/decision_eval.jsonl` — 30 rows (6 each decision/pattern/concept/howto/reference), 114 labels (choice 42, noul 42, score 30).
  - Harness: `apps/wm-core/tests/support/decision_eval.rs`; real run `decision_eval.rs` (`#[ignore]`, feature `decision`).
  - Models (checksum-pinned in `.wm/models.json`): `gliner2.5-small` (74M / 282 MiB), `gliner2.5-decide` (DeBERTa-v3-large / 1.95 GB).
  - Env: `WM_EVAL_SERIALIZATION` / `WM_EVAL_PHRASING` / `WM_EVAL_SCORE_DECODE` / `WM_EVAL_MODEL_VARIANT` / `WM_EVAL_LIMIT`.

  ## Results

  **Per-primitive majority baseline vs the best model (Decide, key-facts + label-descriptions):**

  | primitive | n | baseline | Decide | Δ |
  |---|--:|--:|--:|--:|
  | choice | 42 | 0.500 | **0.524** | **+0.024 (beats)** |
  | noul | 42 | **0.762** | 0.524 | **−0.238** |
  | score | 30 | 0.600 | 0.400 | −0.200 |

  The model **edges the constant on `choice` but collapses on `noul` and `score`**, where the constant is nearly free. That is the refined story: the loss is concentrated in the calibration-heavy primitives — exactly where labelled data would help.

  **Serialization** (`small`, `raw`, n=30): key-facts 0.430 / 7.1 s · trimmed 0.412 / 14.6 s · prose 0.404 / 27.0 s · tagged-prose 0.395 · compact-json 0.386 / 34.9 s.

  **Phrasing** (`small`, `key-facts`): raw 0.430 · **label-descriptions 0.474** · few-shot 0.447.

  **Cumulative-ordinal `score` decode** (small, key-facts + label-descriptions): **worse** — score acc 0.233 → **0.167** (ECE 0.278 → 0.317), overall 0.474 → **0.456**. The emulated ordinals have no usable ordering signal; the lever does not help.

  **Uncertainty (Wilson 95% CI, n=114):** key-facts 0.430 [0.343, 0.522] · label-desc 0.474 [0.384, 0.565] · decide 0.491 [0.401, 0.582] · baseline 0.623 [0.531, 0.706]. The serialization/phrasing/decide spreads (≈2–5 labels) are **within fixture noise**; the gap to baseline (15 labels) is not.

  ## Answers to the driving question

  - **Does gliner "support" YAML/JSON?** No — it is a flat-token encoder; it tokenizes whatever text you pass. **JSON syntax buys nothing and costs most** (`compact-json` 0.386, slowest), so do **not** switch to compact JSON.
  - **Input format**: differences across serializations are within noise — treat serialization as a **latency** lever (key-facts ~3.8× faster than prose), not an accuracy lever.
  - **Accuracy levers**: `label-descriptions` (gliner's trained channel) is the best phrasing; **model scale/specialization does not rescue the zero-shot domain mismatch** (6.5× params bought +0.017).

  ## Verdict

  **Non-viable as-is.** `Decide` improves only +0.017 over the best `small` config (within noise) at ~9.6× the latency of the same config (`118.5 s/row` vs `12.4 s/row`; vs `small` key-facts-raw `7.1 s` it is 16.7×), and still trails the baseline by 15/114.

  ## Why (caveats)

  - Zero-shot on domain/subjective questions with **no training**; calibration is poor (ECE 0.11–0.32). The model wins `choice` but loses `noul`/`score`.
  - `score` (emulated ordinal) is the weakest primitive, and the cumulative decode didn't help.
  - CPU latency is impractical (7 s–118 s/row).

  ## Recommendations

  1. **Keep deterministic heuristics** for these decisions; leave the `decision` feature off by default.
  2. If revisited, the evidence points to **fine-tuning on WM labels** (the losses concentrate in `noul`/`score`, where labelled data bites) — training was out of the 0.6 scope.
  3. Preserve the harness + fixture for future model evaluation.

  ## Reproducibility

  Config lines and per-config results JSONs were emitted by the harness (`wm_decision_eval_<model>_<serialization>_<phrasing>[_<decode>].json` in the OS temp dir). Re-run any cell with `cargo test -p wm-core --features decision --test decision_eval real_model_eval -- --ignored --nocapture` and the `WM_EVAL_*` env vars.

  ## References

  - @doc/specs/improve-gliner-decision-quality
  - @doc/specs/eval-gliner-typed-decisions
  - @doc/concepts/typed-decision-application-map
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
