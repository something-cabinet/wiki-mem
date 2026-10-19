---
title: 'P0: security_test deletes the developer''s installed ONNX model via wm_model remove'
type: task
id: "wiki:tasks:security-test-deletes-the-developers-installed-model"
status: done
tags: [bugfix, test, security, model, p0]
priority: urgent
acceptance_criteria:
  - {text: "No test can delete a model from the developer's real $HOME/.wm/models, proven by running the suite with a sentinel directory present", checked: false}
  - {text: "The model cache is injectable (config or explicit parameter) rather than read from ambient HOME by the wm_model tool", checked: false}
  - {text: "wm001_remove_registry_model_still_succeeds still proves that a valid registry name passes the segment guard, using a fixture directory", checked: false}
  - {text: "cargo test --workspace passes twice in a row on a machine with the model installed", checked: false}
implementation_notes: |-
  Fixed in e392678 (injectable model cache root) and c266578 (degradation test controls its own model).

  Verification: cargo test --workspace run twice in sequence - exit=0 both times, 42 suites ok, 0 failed,
  with ~/.wm/models/bge-small-en-v1.5/model.onnx at 133,093,490 bytes before and after each run. The suite is
  now order-independent and non-destructive.

  wm_model accepts an optional models_dir for list, download and remove; the default stays $HOME/.wm/models.
  security_test removes a registry-named model from a fixture root, and a sentinel test proves the real cache
  survives. semantic_test's degradation case now builds its engine against a model name that cannot exist.
---

**Severity: P0 - a test deletes data outside the repository.**

`packages/wm-tools/tests/security_test.rs:44-55`, test `wm001_remove_registry_model_still_succeeds`,
invokes the in-process `wm_model` tool with `{ action: remove, name: bge-small-en-v1.5 }` and asserts
`status == removed`. The tool resolves its cache with `models_dir()` in
`packages/wm-tools/src/tools/model.rs:12-17`, which reads the ambient `$HOME` (`$HOME/.wm/models`).

So running `cargo test --workspace` on a developer machine **deletes the real installed model** from
`~/.wm/models/bge-small-en-v1.5`.

**Observed 2026-10-19.** After installing bge-small-en-v1.5 (133 MB weights, 711 KB tokenizer verified),
`cargo test --workspace` passed the onnx semantic tests in one run and failed them in the next with
`embedding model 'none' not loaded`, while `~/.wm/models/bge-small-en-v1.5` had disappeared. Test binaries run
concurrently, so whether `semantic_test` sees the model depends on whether `security_test` ran first - the
gate is order-dependent and self-destroying.

**Why it matters beyond one test.** This is not a flake to retry. Any CI or agent run that reaches this test
destroys a 134 MB asset and silently changes the outcome of every other model-dependent test in the same
invocation. It also means a green `cargo test --workspace` cannot be trusted on a developer machine.

**Fix direction.** Make the model cache injectable rather than ambient: resolve it from an explicit input
(engine/config or a parameter) so tests can point it at a temp directory, and have the security test assert
against that directory. Deleting a registry model to prove that deletion is reachable is acceptable only when
the deletion target is a fixture.