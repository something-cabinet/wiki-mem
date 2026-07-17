---
title: "No Comments in Project Code"
type: rule
status: active
category: naming
rationale: "Comments rot, drift from code, and create false confidence. Named functions, descriptive variables, and self-documenting patterns are refactor-safe and always up to date."
example: "Extract a named function `validate_transition()` instead of `// validate state transition`."
anti_pattern: "Inline comments explaining what code does (// increment counter, // check if valid)"
---
