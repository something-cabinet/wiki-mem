---
id: zuj58f
title: Page CRUD + Source State Machine
status: done
priority: high
labels:
  - from-spec
  - go-mode
  - crud
createdAt: '2026-06-15T11:31:23.040Z'
updatedAt: '2026-06-15T14:04:00.129Z'
timeSpent: 0
spec: specs/local-knowledge-engine-rust
fulfills:
  - AC-11
  - AC-20
---
# Page CRUD + Source State Machine

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
page.create/get/update/delete/list, source.add (copy + hash + registry), source.process (CAS transition + orphan recovery), source.complete (state + log.md + rebuild trigger), source.verify (staleness), source.discover (scan configured dirs), source.list, sequential file write channel, orphan timer recovery at startup
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Page CRUD: page.create/get/list with path-based ID resolution. Source state machine: source.add (copy + hash + registry), source.process (CAS pending/stale → processing, orphan 30min timeout), source.complete (auto-appends log.md, triggers rebuild), source.verify (hash comparison staleness), source.list (filter by state), source.discover (walkdir scan of configured dirs, dedup by hash). Sequential file write channel pattern used. Orphan timer recovery function.
<!-- SECTION:NOTES:END -->

