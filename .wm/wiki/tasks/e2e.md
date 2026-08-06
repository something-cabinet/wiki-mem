---
id: wiki:tasks:e2e
title: Remaining E2E and Test Coverage
type: task
status: todo
priority: medium
tags: [testing, e2e, coverage]
acceptance_criteria:
  - text: "All listed P1 tests are implemented and pass (TC-1.3, TC-1.6, TC-1.7, TC-1.8, TC-1.9, TC-2.3, TC-2.5, TC-2.9, TC-2.11, TC-2.12, TC-3.3, TC-4.4, TC-4.7, TC-10.1, TC-10.3, TC-13.1, TC-17.1)"
  - text: "All listed P2 tests are implemented and pass (TC-3.4, TC-6.6, TC-9.4)"
  - text: "cargo test -p wm-core --lib and cargo test -p wm-core --test mcp_test both pass with no failures"
---
id: wiki:tasks:e2e

## Overview

Improve test coverage for areas not yet covered by E2E or integration tests.

## Remaining Coverage Gaps

### P1 Tests Not Yet Implemented
- TC-1.3: Page::meta_mut() mutation round-trip (unit)
- TC-1.6: MemoryLayer::as_str() and Default (unit)
- TC-1.7: PermissionPreset deserialize all variants (unit)
- TC-1.8: RecencyModel deserialize all variants (unit)
- TC-1.9: EdgeType::Custom serialization round-trip (unit)
- TC-2.3: Unknown action returns INVALID_ACTION (MCP)
- TC-2.5: wm_page.link with all 17 edge types (MCP)
- TC-2.9: wm_memory.promote project->global (MCP)
- TC-2.11: wm_decision.create with ADR fields (MCP)
- TC-2.12: wm_template.run with template refs (MCP)
- TC-3.3: migrate with no .wm/memory/ dir (unit)
- TC-4.4: FSRS compaction >100 old versions (unit)
- TC-4.7: compute_field_changes added field (unit)
- TC-10.1: parse_edge_type_flexible all aliases (unit)
- TC-10.3: path_to_id and reverse resolution (unit)
- TC-13.1: ToolError::to_json() includes hint (unit)
- TC-17.1: TriggerEvent::from_str all variants (unit)

### P2 Tests Not Yet Implemented
- TC-3.4: migrate with invalid JSON (unit)
- TC-6.6: Template prompts -> user interaction (unit)
- TC-9.4: PageType with disallowed status in frontmatter (E2E)

## Execution
- Run `cargo test -p wm-core --lib` to verify unit tests
- Run `cargo test -p wm-core --test mcp_test` to verify MCP tests
