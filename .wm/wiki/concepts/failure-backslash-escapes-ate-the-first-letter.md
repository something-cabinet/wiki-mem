---
title: 'Failure: backslash escapes ate the first letter of words in two specs'
type: concept
id: "wiki:concepts:failure-backslash-escapes-ate-the-first-letter"
status: draft
relates_to:
  - {type: references, target: wiki:tasks:add-wm-cli-migrate-migrate-the-whole-wiki-with-versioned-records}
---

schema_version: 1
state: |-
  ## What went wrong

  Two specs, web-server-build-serve and e2e-migration, could not be migrated to records. Their text held control characters: a backspace where the word browser or build began, a bell where apps began, and a carriage return where rust-embed began. In one place a newline replaced the n of ng build.

  ## Root cause

  Text containing backslash sequences such as \b, \a, \r and \n was passed through something that interprets escapes, such as a shell echo or printf, when the page was written. The escape consumed the following letter and left a control character in its place. The YAML record parser then rejected the page.

  ## Prevention

  - migrate lists such pages as skipped with the reason and exits non-zero instead of guessing.
  - Repair is a byte-level edit that maps each control character back to its letter, checked against the surrounding word.
  - Write page text from a file or stdin, never through echo or printf with backslashes in the text.

  ## Time lost

  Small. The corruption was found because migrate reports every skipped page.
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