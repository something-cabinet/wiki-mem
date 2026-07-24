---
id: wiki:specs:stress-scale-tests
title: Stress and Scale Tests
type: spec
status: done
tags: [testing, stress, performance]
relates_to:
  - {type: references, target: wiki:tasks:review-blocking-async-fjadra-layout}
---
id: wiki:specs:stress-scale-tests

## Overview

Add stress and scale tests to ensure the WM engine handles larger workloads without regressions.

## Test Cases

### TC-14.1: 1000 page graph rebuild
Create 1000 pages with varied content, run `wm index rebuild`, verify all 1000 appear in the graph and rebuild completes in <5s on modern hardware.

### TC-14.2: Search across 10K documents
Create 10,000 doc sections (from 1000 pages with 10 sections each). Search for a specific term. Verify results returned in <500ms.

### TC-14.3: Concurrent MCP connections
Spawn 10 concurrent MCP clients, each creating pages simultaneously. Verify no crashes, no data corruption, all pages exist after.

### TC-14.4: Version compaction
Create a task, make 500 rapid field updates. Verify the version file stays under 100KB after FSRS compaction.

### TC-14.5: Memory entry scale
Create 1000 memory entries via the API. Verify `wm_memory.list` returns all entries in <200ms.

## Implementation Notes

- Use `#[ignore]` on stress tests so CI doesn't run them on every commit
- Tag them with `#[cfg_attr(not(feature = "stress"), ignore)]` behind a feature flag
- Use `std::time::Instant` for timing assertions

## Acceptance Criteria
- [ ] AC-1: 1000 page graph rebuild <5s
- [ ] AC-2: 10K doc search <500ms
- [ ] AC-3: 10 concurrent MCP connections stable
- [ ] AC-4: Version compaction <100KB after 500 updates
- [ ] AC-5: 1000 memory entries list <200ms
