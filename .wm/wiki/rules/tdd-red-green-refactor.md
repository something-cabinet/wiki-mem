---
title: TDD — Red-Green-Refactor
type: rule
id: wiki:rules:tdd-red-green-refactor
status: active
category: workflow
rationale: "Tests define behavior before implementation commits to a design. The RED step proves the test catches the failure; a test that passes before implementation is worthless. Small GREEN steps localize failures; the REFACTOR step keeps code clean because correctness is continuously verified."
tags:
- rule
- testing
- tdd
- workflow
relates_to:
  - {type: part_of, target: wiki:core:CONVENTIONS}
---

## Rule: Test-First Development (Red → Green → Refactor)

All implementation work follows the TDD cycle. Write a failing test first, make it pass with the minimal change, then refactor to the cleanest structure. Never write implementation without a test that defines the expected behavior.

### The Cycle

1. **RED** — Write a test that describes the desired behavior. Run it; confirm it fails for the right reason (the feature is missing, not a test harness bug).
2. **GREEN** — Write the minimal implementation to make the test pass. Do not add features the test doesn't ask for.
3. **REFACTOR** — Clean up the code (naming, structure, duplication) while keeping the test green.

### Why

- Tests define behavior before implementation commits to a design — avoids building the wrong thing
- The RED step proves the test catches the failure; a test that passes before implementation is worthless
- Small GREEN steps localize failures — when a test breaks, the change that broke it is small and recent
- The refactor step keeps the codebase clean because correctness is continuously verified, not assumed

### What counts as "the test"

- Rust: unit tests in the same file, integration tests in `tests/`
- Angular: component tests using `MockEngineService`, service unit tests
- Behavior-level tests preferred over implementation-detail tests (test the contract, not the internals)

### Enforcement

- New features/bug fixes must include a failing test written before the implementation (visible in the commit or PR order)
- Bug fixes: write a regression test that reproduces the bug first (RED), then fix (GREEN), then clean up (REFACTOR)
- When a fix is time-boxed and a test genuinely cannot be written first, note the gap explicitly in the task notes — do not silently skip
- Refactors that change no behavior must be covered by the existing test suite staying green

### Exceptions

- Pure documentation/config/boilerplate changes with no behavior to test
- Code generation output (the generator's tests cover it)
- Throwaway scratch/experiments never merged to master