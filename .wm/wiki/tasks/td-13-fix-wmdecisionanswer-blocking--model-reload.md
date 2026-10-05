---
title: TD-13 Fix wm_decision.answer blocking + model reload
type: task
id: "wiki:tasks:td-13-fix-wmdecisionanswer-blocking--model-reload"
status: done
priority: high
tags: [from-review, spec:typed-decision-doc-format, p1, runtime]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "wm_decision.answer no longer blocks the MCP runtime thread (async + spawn_blocking)"
  - text: "The model backend is loaded once and cached, not per call"
  - text: "model_download streams/hashes files instead of buffering ~300MB into memory"
  - text: "Tests updated/added; cargo check/clippy clean"
---

Review P1-1: wm_decision.answer blocks the MCP server and reloads the ~300MB model per call. Make it async (register_typed_async + spawn_blocking), cache the backend (keyed by model dir + manifest revision), and stream-hash downloads.