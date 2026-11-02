---
title: CLI commands must exit non-zero when the tool fails
type: spec
id: "wiki:specs:cli-commands-must-exit-non-zero-when-the-tool-fails"
status: approved
tags: [cli, tool-reliability, spec]
acceptance_criteria:
  - text: Every call site that swallows a tool error now propagates it and the process exits 1
  - text: Success output of every command is unchanged and the golden characterization cases pass
  - text: Each command family has a CLI test asserting a non-zero exit and the error on stderr
---

schema_version: 1
state: |-
  ## Goal

  A wm-cli command whose underlying tool call fails must print the error and exit non-zero, so scripts and agents can tell failure from success.

  ## Verified current state

  - 29 call sites in 10 files under apps/wm-cli/src match the error with eprintln and then return Ok(()), so the process exits 0 (graph, page, search, source, time and others).
  - index rebuild and index embed were fixed first: they now propagate the error and exit 1.
  - Violates @wiki/rules/durable-write-or-error for the write commands (page create, time start).

  ## Change

  Replace each match-and-eprintln with propagation of the tool error through main, which prints it and exits 1. Keep output formats unchanged on success.

  ## Verification

  - Golden characterization cases keep passing.
  - One CLI test per command family runs a failing call and asserts a non-zero exit and the error on stderr.
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
  - id: needs_tasks
    type: noul
    instructions: This spec requires one or more task pages.
answers:
  kind: system
  scope: component
  needs_tasks: true