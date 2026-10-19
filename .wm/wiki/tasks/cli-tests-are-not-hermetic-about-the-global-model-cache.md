---
title: 'CLI tests are not hermetic: a populated global model cache makes every spawned wm-cli load the ONNX model'
type: task
id: "wiki:tasks:cli-tests-are-not-hermetic-about-the-global-model-cache"
status: done
tags: [bugfix, test, performance, hermeticity]
priority: medium
acceptance_criteria:
  - {text: "Spawned wm-cli processes in tests resolve their model cache to the test project, not the developer's $HOME", checked: false}
  - {text: "cargo test -p wm-core --test cli_test completes well inside the 120s per-invocation limit on a machine with the model installed", checked: false}
  - {text: "The suite yields identical results whether or not ~/.wm/models/bge-small-en-v1.5 exists", checked: false}
  - {text: "No production behaviour changes; the fix is confined to the test harness", checked: false}
implementation_notes: |-
  Fixed in commit 2fb65a5. spawn_cli sets HOME and USERPROFILE to the per-test project, so the model cache
  resolves to <project>/.wm/models, which does not exist, and the engine uses the noop embedder.

  Measured after the fix: cargo test -p wm-core --test cli_test = 17 passed in 0.91s (was over 120s with one
  failure); cli_characterization_test = 1 passed in 2.15s.

  No production file touched. The eager-load cost in production remains open as a separate concern.
  Verified: cli_test 17 passed in 0.91s with the model installed, down from over 120s with one failure.

  Measured after the fix: cargo test -p wm-core --test cli_test = 17 passed in 0.91s (was over 120s with one
  failure); cli_characterization_test = 1 passed in 2.15s.

  No production file touched. The eager-load cost in production remains open as a separate concern.
---

**Finding.** The wm-core CLI test suite is not hermetic with respect to the developer's global model cache.
`setup_test_project` pins `embedding.model_name: "bge-small-en-v1.5"`
(`apps/wm-core/tests/helpers/setup.rs:29-31`), and `init_embedder` resolves the model cache from `$HOME`
(`apps/wm-core/src/engine/main_engine_factory.rs:25-28`). So whether a test run loads a 133 MB ONNX model
depends on the machine, not on the repository.

**Observed 2026-10-19.** Before the model was installed, `cargo test --workspace` ran the `cli_test` binary in
about 1.1s. After installing bge-small-en-v1.5 into `~/.wm/models`, the same binary exceeded the harness's
120-second per-invocation limit (`apps/wm-core/tests/helpers/cli.rs:93`) and
`test_health_fix_stubs_and_deletes` began failing, while still passing in isolation in under a second.

**Mechanism.** Each of the 17 tests spawns `wm-cli`; with the model present every spawn pays the eager
`EmbeddingModel::load`, saturating CPU and memory until sibling invocations hit the timeout. Measured:
`wm-cli status` takes about 3.46s with the model installed.

**Fix direction.** Point the spawned CLI's `HOME` (and `USERPROFILE`) at the per-test project directory, so
the model cache resolves to `<project>/.wm/models`, which does not exist, and the engine falls back to the
noop embedder. Tests must give the same result whether or not the developer has a model installed.

**Separate product concern, not fixed here.** Eager model loading makes every CLI command pay the load cost
on machines that legitimately have the model. That is a real UX regression independent of testing, and it
collides with @wiki/specs:engine-state-decomposition, which moves the embedder into `IndexState`. Record it
for a later ruling rather than bundling it into a test fix.