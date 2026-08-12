---
title: Specialist result transmission can silently fail
type: memory
id: wiki:memory:agent-empty-result-transmission
status: active
tags: [orchestration, tooling, workflow]
---

## Pattern

Specialist (oracle/fixer) sessions can complete their work — context-read lists prove files were read — while the final result message comes back EMPTY (network/transmission layer). 2026-08-12 session: 6 consecutive empty oracle transmissions across two sessions; the boards later showed both as completed/reconciled.

## How to handle

- Do NOT burn repeated identical reissues (discipline: adjust scope or stop after ~2).
- Check the tree (git status/diff) before re-dispatching — a "failed" lane may have landed full work; a lane that wrote nothing is verified-empty and safe to re-dispatch (or orchestrator-execute).
- Run the review gate's checklist yourself when the reviewer's delivery is dead: leftover scans (rg), read the changed files, verify the suite — document as "gate executed with blueprint checklist, reviewer delivery unavailable".
- Keep the deepwork file as the durable gate record: evidence + rulings + verdict, with the transmission failure noted.
- Sessions become resumable once reconciled — a compact resume ("deliver your verdict, under 400 words") is worth one try after the board clears.

