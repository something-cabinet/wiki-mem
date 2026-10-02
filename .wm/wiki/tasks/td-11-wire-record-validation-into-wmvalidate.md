---
title: TD-11 Wire record validation into wm_validate
type: task
id: "wiki:tasks:td-11-wire-record-validation-into-wmvalidate"
status: done
priority: high
tags: [from-spec, spec:typed-decision-doc-format, engine, gap]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "wm_validate.check runs the record-envelope validator on record-bearing pages"
  - text: "Malformed records (FR-5) are reported with page + field"
  - text: "Tests cover validate wiring"
relates_to:
  - {type: relates_to, target: wiki:specs:typed-decision-doc-format}
---

Follow-up from TD-06: `wm_validate.check` does not yet run `validate_record` on record bodies (FR-5/AC-3 gap). Wire the record-envelope validator into the validate tool so malformed records are caught.