---
title: Test-only call sites hide unwired features
type: memory
tags: [testing, wiring, dead-code, review]
status: active
---

A function called only from tests satisfies its acceptance criteria while being unreachable in the shipped binary. The suite goes green and the feature does not exist.

Hit 2026-08-17 implementing code-index freshness: `refresh_if_stale` was written, tested, and green — but no production call site existed, so the CLI path it was built for still answered from a stale index. Spec AC-1.1 was "met" by a test calling the function directly. Fixed by calling it from `graph::code_edges::load_code_graph`, then adding a test that exercises the read path rather than the function. Same pass found `index_lag_seconds` with zero callers; it was deleted rather than left as decoration.

Practice: for every new public function, name the production call site before writing the test. In review, ask of each added symbol "what reaches this in a real run?" — if the answer is only a test, it is either unwired or dead, and both are defects. This is the runtime twin of the `#[allow(dead_code)]` masking failure in wiki:core:critical-patterns: there the compiler was silenced, here the test suite provides the false confidence.

Cheap detection: `rg '\bfn_name\b' --glob '!*test*'` after implementing. If only the definition matches, it is not wired.