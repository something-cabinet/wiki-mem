---
title: "FR-1: Rename files to pattern convention"
type: task
status: todo
tags: [design-pattern-alignment, refactor]
task_data:
  acceptance_criteria:
    - text: "Every .rs file under src/ ends with a role suffix (Model/Service/Helper/Constant/Repository/Builder/Factory/Proxy/Mediator)"
      checked: false
    - text: "cargo build --all-features succeeds"
      checked: false
    - text: "cargo test passes same count"
      checked: false
  estimate: 4
  difficulty: "medium"
---
