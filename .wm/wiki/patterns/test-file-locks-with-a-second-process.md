---
title: 'Pattern: Test file locks with a second process'
type: pattern
id: "wiki:patterns:test-file-locks-with-a-second-process"
status: draft
relates_to:
  - {type: references, target: wiki:decisions:open-vectors-db-per-operation}
---

schema_version: 1
state: |-
  ## Problem

  A test that opens the same database file twice inside one process does not prove anything about locking. turso dedupes one `Database` per file in a process, and POSIX file locks belong to the process. Tests for "this code must not hold the lock" passed against code that held it forever. The second open only fails when it comes from another process.

  ## Solution

  Re-run the current test binary as a child process and let the child hold the resource.

  1. Add an `#[ignore]` test that acts as the child. It reads its mode and directory from environment variables, takes the lock, prints a ready line, and waits for stdin to close.
  2. In the parent test, spawn `std::env::current_exe()` with `--ignored --exact <child test name> --nocapture`, the environment variables, piped stdin and stdout.
  3. Read stdout until the ready line, run the code under test, then drop the child's stdin and wait for it.
  4. Use a short, fixed hold time in the child when the test needs the lock to release on its own (for example to check a retry).

  The failure message must come from the lock itself (`File is locked by another process`). If the failure is "never became ready", the harness is broken, not the code.

  ## When to Use

  Any test about file locks, single-writer guarantees or lock release timing.

  ## When Not to Use

  Plain unit tests of lock-free logic. The child costs a process start, about 0.1 s here.

  ## Evidence

  In `packages/wm-embed/src/services/vector_store_service.rs`, three tests failed with the lock error before the per-operation change and pass after it. The in-process version of the same tests passed on both sides.

  ## Related

  - `wiki:decisions:open-vectors-db-per-operation`
  - #146

questions:
  - id: problem_kind
    type: choice
    instructions: What kind of problem does this pattern solve?
    options:
    - architecture
    - api-design
    - data-model
    - error-handling
    - performance
    - testing
    - ui
    - tooling
    - workflow
  - id: preconditions_required
    type: noul
    instructions: This pattern requires specific preconditions to be met.
  - id: complexity
    type: score
    instructions: How complex is applying this pattern?
    levels:
    - trivial
    - simple
    - moderate
    - complex
  - id: language_specific
    type: noul
    instructions: This pattern is specific to a programming language.
answers: {}