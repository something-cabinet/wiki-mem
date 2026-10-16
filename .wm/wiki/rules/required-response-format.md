---
title: Required response format - Goal/result, Key details, Claim, Next action
type: rule
id: "wiki:rules:required-response-format"
status: active
tags: [rule, response-format, output, process]
---

schema_version: 1
state: |-
  ## Rule

  Every response is exactly four sections, in this order, with nothing else. Plain markdown only: no JSON block, and no fence around the response.

  ```markdown
  # Goal/result:
  - <what was accomplished>

  # Key details
  - <supporting context, refs, validation, gaps>

  # Claim
  1. <claim>
  - Reason: <why the claim holds>
  - Evidence: <command, file:line or measurement>
  - Confidence: <high | medium | low>

  2. <claim>
  - Reason: <why the claim holds>
  - Evidence: <command, file:line or measurement>
  - Confidence: <high | medium | low>

  # Next action
  1. <short imperative action>
  2. <short imperative action>
  ```

  ## Section contract

  ### `# Goal/result:`
  One or more bullets stating what was accomplished. Verifiable outcomes only, no process narration.

  ### `# Key details`
  One or more bullets carrying everything a reader needs in order to trust or reproduce the result: file paths, task and spec ids, decisions taken, validation results, assumptions, and known gaps. The `# Key details` section MUST NOT restate the claims.

  ### `# Claim`
  A numbered list. Each entry is one distinct claim, followed by three sub-bullets in this exact order:

  - **Reason** - why the claim holds. Required for every claim.
  - **Evidence** - the command, the `file:line` reference, or the measurement that substantiates it. Write `None` when the claim rests on reasoning alone rather than on inspection.
  - **Confidence** - exactly one of `high`, `medium`, `low`. Required for every claim.

  Claims and details are disjoint: a fact belongs in one section, never both. When a claim is later found false, the correction is a new claim in the same section, not a silent edit of the original.

  ### `# Next action`
  A numbered list of one or more short, imperative, verb-first actions. Indices start at `1` and increment by one, so a reader can reply with just a number. No trailing rationale, no nested lists.

  ## Enforcement

  Review-enforced. A response is in violation when it:

  - omits any of the four sections, or emits them out of order
  - carries a claim with no `Reason`, no `Confidence`, or a `Confidence` outside `high | medium | low`
  - wraps the response in a JSON block or a code fence
  - restates a claim inside `# Key details`

  Severity: required.

  ## Why

  Four named sections give every fact one home: the outcome, the supporting context, the claims with their confidence, and the requested actions. Rendering the confidence inline with each claim keeps the strength of a statement attached to the statement rather than pooled in a separate block, so a reader cannot mistake a low-confidence inference for a verified measurement. An indexed `next_action` list lets a reader reply with just the number of the action they want.

  ## Exceptions

  None.

questions:
  - id: enforcement
    type: choice
    instructions: How is this rule enforced?
    options:
    - ci-enforced
    - tool-enforced
    - review-enforced
    - convention-only
  - id: severity
    type: score
    instructions: How severe is a violation of this rule?
    levels:
    - advisory
    - recommended
    - required
    - blocking
  - id: has_exception
    type: noul
    instructions: This rule has documented exceptions.
  - id: applies_to
    type: choice
    instructions: Which surfaces does this rule apply to?
    multi: true
    options:
    - code
    - tests
    - docs
    - configuration
    - workflow
    - security
answers: {}
