---
id: wiki:decisions:design-pattern-alignment-constants
title: "Decision: Constants in Dedicated Files"
type: decision
status: approved
tags: [decision, constants, static]
decision:
  context: "Static data (constants, OnceLock, LazyLock, RustEmbed) is currently scattered across model and service files. A model file for SkillAssets contains a RustEmbed derive. A regex LazyLock sits in a parser file."
  options:
    - "Keep constants inline where used"
    - "Extract to dedicated XxxConstant.rs files"
  rationale: "Constants are a different concern from models and services. They rarely change, don't participate in business logic, and grouping them makes it easy to audit what statics exist in the system."
  outcome: "All const, static, OnceLock, LazyLock, RustEmbed items go in XxxConstant.rs files."
---

schema_version: 1
state: |-
  id: wiki:decisions:design-pattern-alignment-constants
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
