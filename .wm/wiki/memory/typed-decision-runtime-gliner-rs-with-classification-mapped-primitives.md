---
title: 'Typed-decision runtime: gliner-rs with classification-mapped primitives'
type: memory
tags: [typed-decisions, gliner-rs, runtime, system-one]
status: active
---

Runtime decision for WM typed-decision records: adopt gliner-rs (GLiNER2.5-small) because no sub-200M System One model has a Rust crate and gliner-rs is Rust-native/candle with a per-label probability API. Map primitives: choice=N-label softmax; noul=two-label false/true; score=ordered-label single-label classification (no ordinal head). Feature-gated `decision` (default off), checksum-pinned download. Full: @doc/decisions/adopt-gliner-rs-as-typed-decision-runtime