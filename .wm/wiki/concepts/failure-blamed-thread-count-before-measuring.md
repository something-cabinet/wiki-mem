---
title: 'Failure: Blamed thread count for a hang before measuring on the reporter''s machine'
type: concept
id: "wiki:concepts:failure-blamed-thread-count-before-measuring"
status: draft
relates_to:
  - {type: references, target: wiki:decisions:open-vectors-db-per-operation}
  - {type: references, target: wiki:tasks:index-rebuild-reports-success-when-vectors-cannot-persist}
---

schema_version: 1
state: |-
  ## What went wrong

  A user reported that `wm-cli index rebuild` froze a strong Windows machine (24 threads, 32 GB). From the code and one macOS run, the cause was presented as ONNX using every core with a 32-section batch. The defaults were changed (`c5a4ee4`: half the cores up to 8, batch 8), and the commit message said the old defaults "stalled machines with many cores".

  The reporter then ran the matrix on the Windows machine with the release that included the lock fix:

  - `index rebuild` took 20.1 s at 4 threads, 17.1 s at 8 and 17.0 s at 24.
  - Peak memory was 2.14 GB in every case.
  - The default run did not hang.
  - The hangs were hard resets (`BugcheckCode 0`, no WHEA, no display driver reset), two of three without a wm run. The user suspects mismatched RAM (2666 and 3200 MHz).

  Smaller mistakes in the same thread: a command (`wm-cli index status`) was put in a request to the reporter without checking that it exists, and `code.db` was said to have the same lifetime lock when its callers already open it per operation.

  ## Root cause

  A hypothesis from the code and a different machine was treated as the cause, and code and commit text followed it before the reporter's measurements existed.

  ## Prevention

  - Ask the reporter for measurements on their platform first, and label anything else a hypothesis.
  - Check that a command or file exists before telling someone to run it.
  - Keep commit messages to measured facts. The commit was reworded before it was pushed.
  - Not all of it was wasted: the same defaults cut a forced re-embed on macOS from 597 s and 2.27 GB to 184 s and 0.90 GB (different command and thread count, one run each).

  ## Time lost

  Several hours of builds and test runs on changes that do not address the reported hang.

  ## Related

  - #146
  - `wiki:decisions:open-vectors-db-per-operation`

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