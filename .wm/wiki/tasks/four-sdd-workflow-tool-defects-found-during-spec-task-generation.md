---
title: Four SDD workflow tool defects found during spec task generation
type: task
id: "wiki:tasks:four-sdd-workflow-tool-defects-found-during-spec-task-generation"
status: todo
priority: high
tags: [bug, tool-reliability, mcp, skills, sdd]
acceptance_criteria:
  - text: "wm_task.create with a spec parameter creates the task-to-spec graph edge, so wm_validate.check reports SDD coverage without a separate wm_page link call, with a regression test asserting the edge exists after create"
  - text: "The wm-plan skill source no longer prescribes bracketed task titles, since a leading bracket serialises to unparseable YAML per wiki:tasks:frontmatter-serializer-quoting-for-task-titles-and-acs"
  - text: "Task status vocabulary is reconciled with the board — either superseded becomes a valid task status or it is removed from the board columnOrder"
  - text: "wm_validate.check accepts the path form documented in the wm-spec and wm-plan skills, or every skill is corrected to use the canonical wiki id form"
  - text: "Tasks implementing a spec are discoverable from the spec side, either via wm_graph.neighbors surfacing incoming edges or a documented alternative"
---

Four defects hit in one wm-spec plus wm-plan run on 2026-08-14 while generating tasks for wiki:specs:code-edge-resolution. Each blocks or misleads the documented workflow.

1. spec parameter does not create the edge. All 10 tasks were created with spec set to wiki:specs:code-edge-resolution. wm_validate.check still reported the spec as having no linked tasks until 10 explicit wm_page link calls with edge type implements were issued. Systemic, not local — an sdd scope validation run reported the same no linked tasks warning for 88 of 113 specs, which suggests almost no spec in this wiki has working task linkage. The wm-plan skill calls this fulfills mapping CRITICAL, so the documented contract is unimplemented.

2. The skill template prescribes a title format that corrupts the task. wm-plan documents titles as bracket slug NN bracket then title. A leading bracket makes the frontmatter title parse as a YAML sequence, so the task store cannot resolve the file — already documented in wiki:tasks:frontmatter-serializer-quoting-for-task-titles-and-acs. Titles were written without brackets or colons to avoid it. Fix the skill source, the serializer, or both.

3. Task status vocabulary disagrees with the board. wm_task.update rejected status superseded with Invalid status for task page, allowed todo, in-progress, in-review, done, blocked, cancelled — yet wm_task board returns columnOrder including superseded, reviewed, approved, accepted, rejected, archived, active and stale. Two absorbed tasks had to be marked cancelled instead, which loses the distinction between abandoned and rolled into a spec.

4. wm_validate.check rejects the documented entity form. The wm-spec skill documents entity as specs slash name; that returns Page not found. Only the canonical wiki colon specs colon name form works. Either the resolver should accept both or every skill reference must be corrected.

Related observation, lower severity — wm_graph.neighbors on a spec returns only its outgoing body references, not the incoming implements edges from its tasks, so which tasks implement this spec is not answerable from the spec side even though validation can see the coverage.