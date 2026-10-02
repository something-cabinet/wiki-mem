---
id: wiki:decisions:design-pattern-alignment-file-name-role
title: "Decision: File Name = Pattern Role"
type: decision
status: approved
tags: [decision, naming, convention]
decision:
  context: "The codebase has files named update.rs, crud.rs, state.rs that reveal nothing about their architectural role. A developer must open the file to understand what it does. Gehenna-app convention encodes the pattern role in the filename."
  options:
    - "Keep current naming (no role suffix)"
    - "Add role suffix (Model, Service, Helper, etc.)"
  rationale: "Chosen role suffix because it makes the architecture visible at a glance in file listings, diffs, and imports. A file named PageUpdateBuilderService.rs tells you it's a Builder-pattern Service for Page updates without opening it."
  outcome: "Every .rs file under src/ MUST end with a role suffix. Barrel files (mod.rs) are exempt."
---

schema_version: 1
state: |-
  id: wiki:decisions:design-pattern-alignment-file-name-role
questions:
  - id: outcome
    type: choice
    instructions: What is the recorded outcome of this decision?
    options:
    - adopted
    - rejected
    - deferred
    - superseded
    - abandoned
  - id: reversibility
    type: noul
    instructions: The decision can be reversed cheaply without data migration or cross-module breakage.
  - id: confidence
    type: score
    instructions: How strong is the recorded justification for the selected outcome?
    levels:
    - low
    - medium
    - high
  - id: impact
    type: choice
    instructions: How wide is the blast radius of this decision?
    options:
    - local
    - component
    - system
    - project-wide
answers: {}
