---
id: wiki:decisions:design-pattern-alignment-model-service-split
title: "Decision: Models vs Services Split"
type: decision
status: approved
tags: [decision, model, service, separation]
decision:
  context: "Some files mix struct definitions with their methods. A single file can have 300 lines of data types mixed with 200 lines of business logic operating on them. This makes both harder to navigate and test."
  options:
    - "Keep struct + methods in one file (current)"
    - "Split: struct in XxxModel.rs, methods in XxxService.rs"
  rationale: "Splitting makes the model file focused on data (derives, serialization, validation) and the service file focused on operations (business logic, composition). Each file has one reason to change."
  outcome: "Struct definitions go in XxxModel.rs. Business logic operating on those structs goes in XxxService.rs. If a type has fewer than 5 associated methods, keep them together."
---

schema_version: 1
state: |-
  id: wiki:decisions:design-pattern-alignment-model-service-split
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
