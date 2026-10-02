---
title: 'Failure: Review gate returned empty result — treated as not reviewed'
type: concept
id: wiki:concepts:empty-review-gate-result
status: reviewed
tags:
- failure
- wm-flow
- review
- orchestration
relates_to:
  - {type: references, target: wiki:tasks:re-run-wave-1-review-gate-t1t4t5--initial-gate-returned-empty}
---

schema_version: 1
state: |-
  ---
  title: 'Failure: Review gate returned empty result — treated as not reviewed'
  type: concept
  id: wiki:concepts:empty-review-gate-result
  status: reviewed
  tags:
  - failure
  - wm-flow
  - review
  - orchestration
  relates_to:
    - {type: references, target: wiki:tasks:re-run-wave-1-review-gate-t1t4t5--initial-gate-returned-empty}
    - {type: references, target: wiki:tasks:apply-oracle-recommendations-from-linus-critique-review}
  ---

  ## What went wrong

  wm-flow Wave 1 (T1/T4/T5, Linus-remediation) dispatched a single Oracle review gate (ora-1 resume) covering all three lanes. The task tool returned an **empty result** — the terminal message contained no findings, no verdicts, no file:line refs. The pipeline state nonetheless treated the gate as \"completed, reconciled\", and without explicit checks the wave could have proceeded to Wave 2 on unreviewed code.

  ## Root cause

  - A delegated task returning empty output is indistinguishable from \"no findings\" unless the orchestrator validates output non-emptiness.
  - Background-job lifecycle (\"completed\") tracks that the agent session ended, not that it produced usable output.
  - Review gates were treated as fire-and-forget: dispatch, wait for completion, assume reviewed.
  - The dead delivery channel (ora-1) returned 2 consecutive empty results; no session rotation policy existed.

  ## Prevention

  1. **Validate review-gate output**: if a review task returns empty/no verdicts/no findings, treat it as NOT reviewed — re-dispatch (possibly to a fresh session with explicit \"you must return findings or state GO explicitly\" instruction) before proceeding to dependent lanes.
  2. **Scope review gates to the lane's actual surface**: the empty gate was asked to review 3 disjoint lanes in one call. When one gate covers many lanes, a partial/empty result is harder to detect — prefer per-lane gates (or per-domain review with explicit per-lane verdict requirement).
  3. **Make acceptance explicit in the prompt**: require \"Verdict per lane: GO / GO-with-findings / NO-GO\" as a mandatory return shape.
  4. **Never reuse a dead delivery channel**: if a session has returned N>=2 empty/trivial results, mark it dead and spawn fresh — do NOT retry to the same session for the same gate.

  ## Proven mitigation (2026-08-14)

  The follow-up session applied all 4 prevention measures:
  - 3 **parallel per-lane reviewers** (T1/T4/T5 independently, fresh sessions, ora-1 blacklisted) instead of one mega-gate.
  - Each prompt mandated the exact `T<N>: [GO | GO-with-findings | NO-GO]` output shape with a \"Never return an empty message\" instruction.
  - All 3 returned non-empty verdicts with file:line findings; zero empty deliveries.
  - The per-lane structure also made it trivial to verify completeness (3 non-empty results vs 1 possible-empty covering 3 lanes).

  ## Time lost

  Original: ~15-30 min (re-run required). Follow-up: 0 — the per-lane mitigation worked on first attempt.

  ## Related

  - @wiki/patterns/verify-tree-before-redispatching-failed-lane — verify before re-dispatching a failed lane; this is the review-gate analog
  - @task-re-run-wave-1-review-gate-t1t4t5--initial-gate-returned-empty
  - @wiki/core/critical-patterns — promoted entry \"Review gates must return non-empty verdicts\"
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
