---
title: 'Spec: test suite simplification'
type: spec
id: wiki:specs:test-suite-simplification
status: draft
---

schema_version: 1
state: |-
  ## Approach

  1. **Tier 1 — unit**: grow in-`src/` tests of pure functions (parser, BM25, graph, ranking). Zero processes.
  2. **Tier 2 — library-level integration**: generalize the security_test pattern — `setup_test_project()` + `EngineState::new` + `register_all_tools` + `registry.dispatch_async`. One in-process engine per test, tempdir-isolated, sub-second. Absorbs mcp_test/cli_test/e2e behavior tests.
  3. **Tier 3 — thin transport E2E** (~10-15 tests): one MCP stdio round-trip, one CLI smoke, cross-token isolation, singleton-refusal, web API contract. One shared DaemonHandle per binary.
  4. **Tier 4 — benchmarks**: criterion, never #[ignore]d wall-clock asserts.
  5. **CI**: three jobs — lib tests (no daemons, no ort), in-process integration, small E2E. nextest with per-test timeout — hang failure class dies by construction.
  6. **Angular**: at least one real component test with MockEngineService per CONVENTIONS.

  ## Constraints

  - Behavior-contract assertions only (no log prose, no key-existence-only checks).
  - No fixed sleep(); readiness polling with deadline.
  - No kill -9 teardown.
questions:
  - id: kind
    type: choice
    instructions: What kind of spec is this?
    options:
    - feature
    - system
    - doc
    - migration
    - experiment
  - id: scope
    type: choice
    instructions: How wide is the scope of this spec?
    options:
    - local
    - component
    - system
    - project-wide
  - id: status_class
    type: choice
    instructions: What lifecycle class is this spec in?
    options:
    - draft
    - reviewed
    - approved
    - superseded
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers: {}
