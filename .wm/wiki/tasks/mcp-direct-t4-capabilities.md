---
title: T4: Capabilities + catalog quality
type: task
status: done
priority: medium
tags: [from-spec, spec/mcp-direct-handlers]
---

## Description

Verify capabilities honesty and tool catalog quality: no `listChanged` advertised, all tools have non-empty descriptions and real input schemas, consolidated action-enum tools enumerate actions in descriptions.

## Acceptance Criteria

- [ ] AC-2: `tools/list` returns all registered tools, each with a non-empty description and non-empty input schema
- [ ] AC-11: `initialize` response advertises tools capability without `listChanged`; no `list_changed` notification ever sent
- [ ] AC-7: `cargo check -p wm-cli -p wm-core -p wm-server` passes clean

## Fulfills

- NFR-5: No `tools.listChanged` capability for static tool set
- NFR-6: `tools/list` has non-empty descriptions/schemas; parity with wm-server
