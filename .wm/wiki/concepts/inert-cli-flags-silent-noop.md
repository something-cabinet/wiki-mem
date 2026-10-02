---
title: 'Failure: Inert CLI flags — acknowledged but never wired'
id: wiki:concepts:inert-cli-flags-silent-noop
type: concept
relates_to:
  - {type: relates_to, target: wiki:concepts:cargo-npm-scoped-output-silent-noop-glob}
  - {type: references, target: wiki:tasks:wm-index-code-output-misleading--report-totals-make---skip-hash-check-force-re-parse}
tags: [failure, cli, ux]
---

schema_version: 1
state: |-
  # Failure: Inert CLI flags — acknowledged but never wired

  ## What went wrong

  `wm index code --skip-hash-check` was accepted by clap and replied with "skip_hash_check flag acknowledged — hash-check behavior is always active", but the boolean was never threaded into `rebuild_code_index`. Users passing the flag expecting a forced full re-index got a silent no-op. The flag existed for UX/API compatibility but changed nothing.

  ## Root cause

  The flag was added to the clap action enum and "acknowledged" with a log line, but the value never reached the underlying function. Acknowledging a flag (logging "got it") is worse than rejecting it: it creates the illusion of control and hides that behavior is unchanged. The bug is invisible to tests that only exercise default paths.

  ## Prevention

  - Every CLI flag must thread into behavior or be removed — an ack log is NOT wiring
  - When adding a flag, add a test that exercises the flag's code path (e.g. force re-parse asserts files_changed > 0)
  - During review, grep the flag name: it should appear in the clap definition AND in the call site of the underlying function
  - If a flag is kept for API compatibility, document that it is a no-op and never log a fake ack

  ## Time lost

  ~30 min to find, wire, and verify — plus user confusion on every `--skip-hash-check` invocation before the fix, and a false "broken index" investigation triggered by the related output bug.

  ## Related

  - @wiki/concepts/cargo-npm-scoped-output-silent-noop-glob — same silent-noop family (glob matched nothing)
  - @wiki/patterns/cli-delta-vs-total-reporting
questions:
  - id: kind
    type: choice
    instructions: What kind of concept document is this?
    options:
    - concept
    - failure-analysis
    - research-report
    - reference-note
  - id: category
    type: choice
    instructions: Which domain category does this concept belong to?
    options:
    - architecture
    - search-retrieval
    - graph
    - parser-format
    - mcp-tooling
    - cli
    - storage
    - embeddings
    - web-ui
    - process
  - id: maturity
    type: score
    instructions: How mature is the understanding of this concept?
    levels:
    - raw
    - exploratory
    - established
    - stable
  - id: code_referenced
    type: noul
    instructions: This concept references concrete code.
answers: {}
