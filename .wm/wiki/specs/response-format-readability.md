---
title: Response Format Readability — Human-Readable Output Under the Tag Envelope
type: spec
id: "wiki:specs:response-format-readability"
status: approved
relates_to:
  - {type: references, target: wiki:rules:required-response-format}
tags: [spec, approved, response-format, readability, hybrid, next-action]
acceptance_criteria:
  - checked: false
    text: 'AC-1: every reply carries a non-empty `next_action` list whose entries each have an integer `index` (from 1), a short imperative `action`, and no `label` (check).'
  - checked: false
    text: 'AC-2: a factual reply still carries a json block with at least one reasoning item and a top-level `confidence` (check).'
  - checked: false
    text: 'AC-3: one reasoning item per distinct claim, each with its own `confidence` (check).'
  - checked: false
    text: 'AC-4: an assumption item carries `reason`; a verified item does not (check).'
  - checked: false
    text: 'AC-5: the markdown presents Goal/result, then Key details, and contains no next-action prose (check).'
  - checked: false
    text: 'AC-6: the response markdown is not fenced or escaped (check).'
  - checked: false
    text: 'AC-7: the json block parses via `jq_validate` (check).'
  - checked: false
    text: 'AC-8: no `why` field appears anywhere (check).'
  - checked: false
    text: 'AC-9: the markdown contains no reasoning, next-action, or confidence text that the json block also carries (check).'
---

schema_version: 1
state: |-
  ## Overview

  Make the mandatory response format readable and enforced. The format is split: a **plain-markdown response** (Goal/result -> Key details) that renders normally, plus a **fenced json block** carrying the machine data including a mandatory **`next_action`**.

  ## Locked Decisions

  - **D1**: Scope is the readability and enforcement of the structured response format.
  - **D2**: **`response` is unstructured plain markdown** — no wrapper, no fence, no escaping.
  - **D3**: The markdown sections are **Goal/result -> Key details** only; the next action is **not** prose.
  - **D4**: A per-item **`reason`** is required when the claim rests on an assumption, a judgement call, or non-obvious evidence; omitted when directly verified or mechanical.
  - **D5**: **One reasoning item per distinct claim**, soft budget about 7; `reasoning` is mandatory with at least one item.
  - **D6**: Envelope history (2026-10-16): tags -> JSON (rejected) -> tags -> fenced JSON -> YAML -> **hybrid adopted** (plain markdown response + fenced json block).
  - **D7**: **`next_action` is a required field** in the json block — a list of one or more entries, each `{ "index": <integer>, "action": "…" }`, indexed from **1**. It is never omitted, and the index makes each action referenceable.
  - **D8**: The json block is **validated and pretty-printed before sending** (`jq`), so the machine contract cannot break on malformed JSON.
  - **D9**: The two parts are **disjoint** — the plain markdown MUST NOT restate anything the json block carries (reasoning, `next_action`, or confidence); those appear only in the json block.
  - **D10**: `next_action` entries carry **no `label`** — the integer index is the identifier, and each `action` must be **short and imperative** (verb-first, no trailing rationale).

  ## Requirements

  ### Functional Requirements

  - **FR-1**: Every response is plain markdown followed by exactly one fenced json block, and nothing else.
  - **FR-2**: The markdown presents Goal/result, then Key details.
  - **FR-3**: The json block contains `reasoning` (mandatory, at least one item, one per distinct claim, each with `claim` and its own `confidence`), `next_action`, and a top-level `confidence`.
  - **FR-4**: `next_action` is **mandatory** and holds one or more entries, each with an **integer `index`** (starting at 1) and a **short, imperative `action`** string; entries carry no `label`.
  - **FR-5**: An item includes `reason` iff its claim rests on an assumption, a judgement call, or non-obvious evidence.
  - **FR-6**: `confidence` is a level only (`high`/`medium`/`low`) — no `why` field.
  - **FR-7**: The json block is valid JSON and is formatted/validated before sending.
  - **FR-8**: The markdown and the json block do not duplicate each other; reasoning, `next_action`, and confidence appear only in the json block.

  ### Non-Functional Requirements

  - **NFR-1**: **Readable** — the response renders as ordinary markdown, unescaped and unfenced.
  - **NFR-2**: **Machine-parseable** — one fenced json block, valid on receipt.
  - **NFR-3**: **Uniform** — same shape for plain replies and skill runs.
  - **NFR-4**: No new tooling beyond a JSON formatter/validator.

  ## Acceptance Criteria

  - [ ] AC-1: every reply carries a non-empty `next_action` list whose entries each have an integer `index` (from 1), a short imperative `action`, and no `label` (check).
  - [ ] AC-2: a factual reply still carries a json block with at least one reasoning item and a top-level `confidence` (check).
  - [ ] AC-3: one reasoning item per distinct claim, each with its own `confidence` (check).
  - [ ] AC-4: an assumption item carries `reason`; a verified item does not (check).
  - [ ] AC-5: the markdown presents Goal/result, then Key details, and contains no next-action prose (check).
  - [ ] AC-6: the response markdown is not fenced or escaped (check).
  - [ ] AC-7: the json block parses via `jq_validate` (check).
  - [ ] AC-8: no `why` field appears anywhere (check).
  - [ ] AC-9: the markdown contains no reasoning, next-action, or confidence text that the json block also carries (check).

  ## Scenarios

  ### Scenario 1: Trivial factual reply
  **Given** a one-line factual answer
  **When** the responder replies
  **Then** the markdown carries the answer, and the json block carries one reasoning item, a `next_action` list, and a top-level `confidence`.

  ### Scenario 2: Next action cannot be omitted
  **Given** a reply where no next step is obvious
  **When** the responder replies
  **Then** `next_action` is still present and states the best available next step (for example "none — work complete").

  ### Scenario 3: Malformed block prevented
  **Given** a draft json block
  **When** it is validated before sending
  **Then** a malformed block is corrected before the response is emitted.

  ## Technical Notes

  The format grammar is normative in `@wiki/rules:required-response-format`. Only the json block is fenced; the response is plain markdown. Validation uses the jq formatter/validator available to the responder.

  ## Open Questions

  - [ ] Should confidence levels be a fixed vocabulary with defined thresholds?
  - [ ] Is about 7 the right soft item budget?
  - [ ] Should the rule's `enforcement` / `severity` / `applies_to` answers be set (currently empty)?
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