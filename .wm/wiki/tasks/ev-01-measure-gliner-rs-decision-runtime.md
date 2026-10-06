---
title: EV-01 Measure gliner-rs decision runtime
type: task
id: "wiki:tasks:ev-01-measure-gliner-rs-decision-runtime"
status: todo
priority: high
tags: [from-spec, spec:eval-gliner-typed-decisions, eval, measurement]
spec: specs/eval-gliner-typed-decisions
acceptance_criteria:
  - text: "Labeled fixture ~30 rows across all types/primitives"
  - text: "Harness computes accuracy/macro-F1/ECE/latency per primitive + overall"
  - text: "Real gliner run executed or precise blocker documented"
  - text: "Report doc with numbers + verdict"
---

Build the eval harness, author the labeled fixture, run gliner2.5-small end-to-end with the decision feature, and report accuracy/ECE/latency per primitive. Per specs/eval-gliner-typed-decisions.