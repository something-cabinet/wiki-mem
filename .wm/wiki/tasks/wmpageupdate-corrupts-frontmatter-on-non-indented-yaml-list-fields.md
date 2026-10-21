---
title: wm_page.update corrupts frontmatter on non-indented YAML list fields
type: task
id: "wiki:tasks:wmpageupdate-corrupts-frontmatter-on-non-indented-yaml-list-fields"
status: done
priority: high
tags: [bug, tool-reliability, frontmatter, wm_page]
acceptance_criteria:
  - text: "Repro: updating a page with a `tags` list corrupts frontmatter (orphaned list items)"
  - text: "set_yaml_value_field/remove_yaml_block handle non-indented YAML list blocks"
  - text: "Regression test for round-tripping tags via wm_page.update"
implementation_notes: 'Fixed. remove_yaml_block/set_yaml_value_field consume non-indented list items; page_update_builder now writes tags and ACs through set_yaml_value_field instead of format!. Regression: set_yaml_value_field_replaces_non_indented_list_block and test_update_page_hostile_tags_and_acceptance_criteria_roundtrip (hostile leading [, colon, quote) pass.'
---

schema_version: 1
state: |-
  Tool reliability (found during RW-01): wm_page.update with a `tags` list corrupts frontmatter — `set_yaml_value_field`/`remove_yaml_block` do not consume non-indented YAML list blocks, leaving orphaned `- item` lines. Workaround used: content-only updates. Reported per rule tool-reliability-bug-tracking.
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