---
title: System One decision-model feasibility for WM — no small Rust model, use custom ort
type: memory
tags: [system-one, decision-model, onnx, ort, feasibility, edge-typing]
status: active
---

Feasibility spike (spec specs/system-one-model-application, report wiki:concepts:system-one-feasibility). Key durable findings: (1) No sub-200M System One decision model has a prebuilt Rust crate — Rust-native crates (laya, gliner-rs, jigor, ollaya) serve models ≥322M or choice-only. (2) The viable small, local, free path is a thin custom `ort` wrapper (WM already depends on ort 2.0.0-rc.12 + tokenizers behind the default-on `onnx` feature in wm-embed) around a sub-200M ONNX decision model: Verdict-small (118M, int8 112 MiB) or Julia-1 (144M, full choice/score/noul, 393 MB RSS documented). (3) Laya (~322M) is the smallest Rust-native System One model but ~2.4 GB peak — too heavy for the target ~8GB CPU-only, no-GPU machine. (4) First prototype recommended: graph edge typing, scoped to classes with ≥30 examples; frontmatter corpus = 280 typed edges (references 196, implements 64; zero extends/depends_on; raw grep ~326 is contaminated by doc examples + duplicated core frontmatter). (5) Per-workflow verdicts: search NO-GO, classification GO (validation only), memory/rule NEEDS-FINETUNE, tool-routing/review NO-GO, edge typing GO.