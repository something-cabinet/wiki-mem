---
title: 'Failure: Review gate returned empty result — treated as not reviewed'
type: concept
id: wiki:concepts:empty-review-gate-result
status: draft
tags:
- failure
- wm-flow
- review
- orchestration
relates_to:
  - {type: references, target: wiki:tasks:re-run-wave-1-review-gate-t1t4t5--initial-gate-returned-empty}
---

## What went wrong

wm-flow Wave 1 (T1/T4/T5, Linus-remediation) dispatched a single Oracle review gate (ora-1 resume) covering all three lanes. The task tool returned an **empty result** — the terminal message contained no findings, no verdicts, no file:line refs. The pipeline state nonetheless treated the gate as "completed, reconciled", and without explicit checks the wave could have proceeded to Wave 2 on unreviewed code.

## Root cause

- A delegated task returning empty output is indistinguishable from "no findings" unless the orchestrator validates output non-emptiness.
- Background-job lifecycle ("completed") tracks that the agent session ended, not that it produced usable output.
- Review gates were treated as fire-and-forget: dispatch, wait for completion, assume reviewed.

## Prevention

1. **Validate review-gate output**: if a review task returns empty/no verdicts/no findings, treat it as NOT reviewed — re-dispatch (possibly to a fresh session with explicit "you must return findings or state GO explicitly" instruction) before proceeding to dependent lanes.
2. **Scope review gates to the lane's actual surface**: the empty gate was asked to review 3 disjoint lanes in one call. When one gate covers many lanes, a partial/empty result is harder to detect — prefer per-lane gates (or per-domain review with explicit per-lane verdict requirement).
3. **Make acceptance explicit in the prompt**: require "Verdict per lane: GO / GO-with-findings / NO-GO" as a mandatory return shape.

## Time lost

Estimated ~15-30 min: the review must be re-run (`wiki:tasks:re-run-wave-1-review-gate-t1t4t5--initial-gate-returned-empty`), and the risk window where Wave 2 could have been dispatched on unreviewed code.

## Related

- @wiki/patterns/verify-tree-before-redispatching-failed-lane — verify before re-dispatching a failed lane; this is the review-gate analog
- @task-re-run-wave-1-review-gate-t1t4t5--initial-gate-returned-empty