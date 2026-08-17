---
title: Frontmatter serializer quoting for task titles and ACs
type: task
id: wiki:tasks:frontmatter-serializer-quoting-for-task-titles-and-acs
status: todo
priority: high
tags: [bug, frontmatter, task-store]
acceptance_criteria:
  - text: "Task and doc frontmatter writers quote title and AC values that YAML would misinterpret (leading [, colons, backslashes)"
  - text: "Round-trip: create task with bracketed title -> file parses -> task resolvable via get"
  - text: "Regression test covers the quoting behavior"
implementation_notes: 2026-08-14 — additional evidence and a second cause. The wm-plan skill's own task template prescribes titles in the form bracket slug NN bracket then title, which is exactly the shape this bug corrupts. Any agent following the documented SDD flow creates unresolvable tasks. During task generation for wiki:specs:code-edge-resolution the bracket prefix was dropped deliberately and colons avoided in both titles and acceptance criteria, so the 10 tasks parse correctly. Fix needs to cover the skill source as well as the serializer, otherwise the serializer fix leaves a template that only works by accident. Tracked alongside wiki:tasks:four-sdd-workflow-tool-defects-found-during-spec-task-generation.
---

Found while creating wm-doc-fix tasks: the task writer builds frontmatter via format! (apps/wm-core/src/mcp/tools/task/mod.rs) writing title and acceptance_criteria UNQUOTED. Titles starting with [ or containing : produce unparseable YAML (sequence / nested mapping), so the task store cannot resolve the file (task not found) until the frontmatter is fixed. Same root family as issue #126: manual frontmatter string-building instead of a shared YAML-aware writer. wm_page uses yaml_helper (page/helpers/yaml_helper.rs) which quotes correctly - task store should reuse the same approach.