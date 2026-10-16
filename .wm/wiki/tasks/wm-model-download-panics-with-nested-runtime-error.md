---
title: wm model download panics with a nested-runtime error instead of fetching the model
type: task
id: "wiki:tasks:wm-model-download-panics-with-nested-runtime-error"
status: todo
tags: [bugfix, cli, model, tool-reliability]
priority: high
acceptance_criteria:
  - {text: "wm model download bge-small-en-v1.5 completes, or fails with an actionable message; it never panics", checked: false}
  - {text: "The blocking HTTP client is never constructed or dropped inside an async runtime context", checked: false}
  - {text: "A test covers the download entry point so the nested-runtime panic cannot return unnoticed", checked: false}
  - {text: "After a successful download, the onnx semantic tests in packages/wm-tools/tests/semantic_test.rs pass", checked: false}
---

**Bug.** `wm model download bge-small-en-v1.5` panics before it fetches anything:

```
thread 'main' panicked at tokio-1.52.4/src/runtime/blocking/shutdown.rs:51:21:
Cannot drop a runtime in a context where blocking is not allowed. This happens when a runtime is dropped
from within an asynchronous context.
ERROR reqwest::blocking::client: Failed to communicate successful startup: Ok(())
```

**Reproduced 2026-10-16** at HEAD 67a8e0a on macOS: `./target/debug/wm-cli model download bge-small-en-v1.5`.
`wm model list` reports the model as cached and available, so the failure is in the download path, not in
model resolution.

**Diagnosis.** A `reqwest::blocking` client is constructed on a thread that is already inside a tokio
runtime, and its internal runtime is then dropped in that async context. The blocking client must be built
and driven on a dedicated thread or outside the runtime, or the async client must be used instead.

**Why it matters.** The ONNX model is an environment prerequisite for the semantic and hybrid search tests,
so a broken download path makes `cargo test --workspace` permanently red on any machine without a
pre-populated model cache. That in turn blocks every task whose acceptance criteria require a green
workspace suite, including the remaining migrations of @wiki/specs:engine-state-decomposition.
Related: @wiki/tasks:cli-surface-defects-found-by-the-step-0-characterization-net carries the requirement
that the onnx tests either pass with a valid model cache or fail naming the attempted path.