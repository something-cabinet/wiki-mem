---
title: health audit --fix and graph export obsidian exit 0 on failure
type: task
id: "wiki:tasks:health-audit---fix-and-graph-export-obsidian-exit-0-on-failure"
status: done
priority: medium
tags: [cli, tool-reliability]
spec: wiki:specs:cli-commands-must-exit-non-zero-when-the-tool-fails
acceptance_criteria:
  - text: health audit --fix exits 1 and names each fix that could not be applied
    checked: true
  - text: graph export obsidian without --out exits 1 with the message on stderr
    checked: true
  - text: A CLI test covers each, and the dry-run and success paths still exit 0
    checked: true
implementation_notes: health_service.rs collects each failed delete, stub and ref rewrite in FixReport.failures, prints the audit as before, then fails with the list; graph_export_renderer.rs bails when obsidian has no --out. Tests in apps/wm-core/tests/cli_exit_code_test.rs failed first (exit 0). The health test makes the tasks directory read-only, so it runs on unix only.
---

schema_version: 1
state: |-
  Follow-up to the CLI exit code task. health audit --fix printed per-item failures to stderr and exited 0, and graph export obsidian without --out printed a message and exited 0. Both now fail the process. See the CLI exit code spec.
questions:
  - id: work_kind
    type: choice
    instructions: What kind of work is this task?
    options:
    - feature
    - bugfix
    - refactor
    - docs
    - test
    - chore
    - migration
  - id: priority
    type: choice
    instructions: What priority is this task?
    options:
    - low
    - medium
    - high
    - urgent
  - id: needs_spec
    type: noul
    instructions: This task depends on a spec.
  - id: has_ac
    type: noul
    instructions: This task has at least one acceptance criterion.
answers: {}