---
title: Page CRUD + Source State Machine
type: task
status: done
tags: [from-spec, go-mode, crud]
priority: high
knowns_id: zuj58f
spec: specs/local-knowledge-engine-rust
fulfills: [AC-11, AC-20]
relates_to:
  - {type: implements, target: wiki:specs:local-knowledge-engine-rust}
---

# Page CRUD + Source State Machine

> **Spec:** `specs/local-knowledge-engine-rust`

> **Fulfills:** AC-11, AC-20

> *Imported from Knowns task `zuj58f`*

# Page CRUD + Source State Machine

## Description


page.create/get/update/delete/list, source.add (copy + hash + registry), source.process (CAS transition + orphan recovery), source.complete (state + log.md + rebuild trigger), source.verify (staleness), source.discover (scan configured dirs), source.list, sequential file write channel, orphan timer recovery at startup


## Acceptance Criteria



## Implementation Notes


Page CRUD: page.create/get/list with path-based ID resolution. Source state machine: source.add (copy + hash + registry), source.process (CAS pending/stale → processing, orphan 30min timeout), source.complete (auto-appends log.md, triggers rebuild), source.verify (hash comparison staleness), source.list (filter by state), source.discover (walkdir scan of configured dirs, dedup by hash). Sequential file write channel pattern used. Orphan timer recovery function.
