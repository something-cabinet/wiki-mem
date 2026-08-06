---
id: wiki:tasks:stress
title: Stress and Scale Tests
type: task
status: todo
priority: low
tags: [testing, stress, performance]
acceptance_criteria:
  - text: "1000-page graph rebuild completes in under 5s and search across 10K documents returns results in under 500ms"
  - text: "10 concurrent MCP connections run without crashes or data corruption"
  - text: "500 rapid version updates keep compacted file size under 100KB"
---
id: wiki:tasks:stress

## Overview

Add stress/scale tests for the WM engine to ensure it handles larger workloads.

## Requirements

- TC-14.1: Create 1000 pages and verify graph rebuild completes <5s
- TC-14.2: Search across 10K documents returns results <500ms
- TC-14.3: 10 concurrent MCP connections — no crashes or data corruption
- TC-14.4: 500 rapid version updates — compaction keeps file size <100KB

## Acceptance Criteria
- [ ] AC-1: 1000 page graph rebuild <5s
- [ ] AC-2: 10K doc search <500ms
- [ ] AC-3: Concurrent MCP connections stable
- [ ] AC-4: Version compaction effective
- [ ] AC-5: All stress tests pass
