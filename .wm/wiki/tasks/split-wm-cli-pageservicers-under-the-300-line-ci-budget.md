---
title: Split wm-cli page_service.rs under the 300-line CI budget
type: task
id: "wiki:tasks:split-wm-cli-pageservicers-under-the-300-line-ci-budget"
status: done
priority: high
tags: [ci, wm-cli, structure, refactor]
acceptance_criteria:
  - text: "apps/wm-cli/src/page/page_service.rs is at most 300 lines"
  - text: "Update payload/input helpers live in a dedicated module with their tests"
  - text: "No CLI behavior change; existing CLI tests stay green"
  - text: "The CI `check` job's file-size step passes"
relates_to:
  - {type: implements, target: wiki:specs:wm-cli-page-service-size-budget}
implementation_notes: Committed as 394017c. CI run 39397172947 passed the wm-cli file-size step (page_service.rs 332 -> 223); the run then failed later at Gate 1 on the pre-existing wm-core ceiling, tracked separately.
---

CI `check` job fails at the `Enforce wm-cli file-size budget` step: `apps/wm-cli/src/page/page_service.rs: 332 lines > 300`. The first failing run is 39353845207 (docs commit, 2026-10-20T01:30Z), so this predates the code-index work; commit 03f4e4f grew the file.

Fix direction: extract the update-payload/input cluster (`read_stdin`, `read_update_payload`, `apply_content_file`, `read_content_file`, `parse_update_payload`, `attach_update_identity`, their constants, and their tests) into a dedicated `page_payload_helper.rs` module. No behavior change.