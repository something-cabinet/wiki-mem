---
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
