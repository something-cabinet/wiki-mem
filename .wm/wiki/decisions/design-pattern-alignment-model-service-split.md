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
id: wiki:decisions:design-pattern-alignment-model-service-split
