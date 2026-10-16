---
title: Required response format — <response> and <confidence> blocks
type: rule
id: "wiki:rules:required-response-format"
status: active
tags: [rule, response-format, output, process]
---


schema_version: 1
state: |-
  ## Rule

  Every response has exactly two parts, in this order, with nothing else:

  1. **The response itself** — plain markdown, with **no wrapper and no fence**; sections in the order **Goal/result -> Key details**.
  2. **A fenced json block** — the machine contract:

  ```json
  {
    "reasoning": [ { "claim": "…", "confidence": "high|medium|low", "reason": "…" } ],
    "next_action": [ { "index": 1, "action": "…" } ],
    "confidence": "high|medium|low"
  }
  ```

  - **reasoning** — mandatory, at least one item; **one item per distinct claim**; each item carries **its own confidence level**.
  - **reason** — optional per item; **required** when the claim rests on an assumption, a judgement call, or non-obvious evidence; omitted when directly verified or mechanical.
  - **next_action** — **mandatory**, a list of **one or more entries**, each `{ "index": <integer>, "action": "…" }`. Indices start at **1** and increment by one, so each action is referenceable (for example "do 2"). There is **no `label` field**: the index is the identifier. Each `action` **MUST be short and imperative** (verb-first, no trailing rationale). Never omitted.
  - **confidence** (top level) — the level only. **No `why` field**; justification belongs in `reason`.
  - **No duplication** — the two parts are **disjoint**. The markdown **MUST NOT restate** anything the json block carries: not the reasoning, not the `next_action`, not the confidence. The markdown carries Goal/result and Key details only.

  ## Pre-send requirement

  Before a response is sent, its json block **MUST be formatted and validated** (for example by passing it through a JSON formatter/validator such as `jq`).

  - If validation fails, correct the block and validate again.
  - A response **MUST NOT be sent** with an unformatted, unvalidated, or malformed block.
  - "Formatted" means pretty-printed and consistently indented, not emitted as one raw line.

  ## Why

  Plain markdown renders normally with no escaping or fencing, while the machine contract stays in one fenced json block. Keeping the parts disjoint means each fact has exactly one home, and an indexed `next_action` list lets a reader reply with just the number of the action they want.

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
