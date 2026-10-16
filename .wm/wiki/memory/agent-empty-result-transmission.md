---
title: Specialist result transmission can silently fail
type: memory
id: wiki:memory:agent-empty-result-transmission
status: active
tags:
- orchestration
- tooling
- workflow
relates_to:
  - {type: references, target: wiki:tasks:wiki-tool-reliability-wmpageupdate-returns-ioerror-on-success}
---

schema_version: 1
state: |-
  ## Pattern

  Specialist (oracle/fixer) sessions can complete their work — context-read lists prove files were read — while the final result message comes back EMPTY (network/transmission layer). 2026-08-12 session: 6 consecutive empty oracle transmissions across two sessions; the boards later showed both as completed/reconciled.

  ## How to handle

  - Do NOT burn repeated identical reissues (discipline: adjust scope or stop after ~2).
  - Check the tree (git status/diff) before re-dispatching — a "failed" lane may have landed full work; a lane that wrote nothing is verified-empty and safe to re-dispatch (or orchestrator-execute).
  - Run the review gate's checklist yourself when the reviewer's delivery is dead: leftover scans (rg), read the changed files, verify the suite — document as "gate executed with blueprint checklist, reviewer delivery unavailable".
  - Keep the deepwork file as the durable gate record: evidence + rulings + verdict, with the transmission failure noted.
  - Sessions become resumable once reconciled — a compact resume ("deliver your verdict, under 400 words") is worth one try after the board clears.

  ## 2026-10-15 — the remedy that works: one bounded re-emit to the same session

  Two empty transmissions occurred this session (a librarian lane and an oracle lane, both reporting `state: completed` with zero text). In both cases the recovery that worked was a **single bounded re-emit request sent to the same session id** — not a respawn, and explicitly not a repeat of the original research.

  Both lanes then returned complete results, and one produced a full architecture verdict it had already computed. Recommended phrasing: *"your previous turn completed but the result arrived empty; re-emit what you established, do not redo the work, fill only the gaps that remain open"*. If that also returns empty, treat the lane as lost rather than looping — the session remains resumable once the board reconciles it.

  Source: @wiki/tasks:wiki-tool-reliability-wmpageupdate-returns-ioerror-on-success
questions:
  - id: layer
    type: choice
    instructions: Which memory layer does this entry belong to?
    options:
    - project
    - global
    - session
  - id: store_or_skip
    type: noul
    instructions: This entry is worth storing as durable memory.
  - id: dedup_action
    type: choice
    instructions: How should this entry relate to existing memory?
    options:
    - new
    - merge
    - supersede
    - skip
  - id: confidence
    type: score
    instructions: How confident is the recorded knowledge?
    levels:
    - low
    - medium
    - high
answers: {}