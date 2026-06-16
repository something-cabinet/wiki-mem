---
name: wm-spec
description: Create a specification document as a wiki page
---

# Create Spec

**Announce:** "Using wm-spec to create spec for [name]."

## Steps

### 1. Create spec page
```
wm_page.create(
  path="wiki/specs/<slug>.md",
  title="<Feature Name>",
  content="## Overview\n\n## Requirements\n\n## Acceptance Criteria\n\n..."
)
```

### 2. Link to related concepts
```
wm_page.update(
  id="wiki:specs:<slug>",
  relates_to=[
    {type: implements, target: "wiki:concepts:<related>"},
    {type: depends_on, target: "wiki:specs:<prerequisite>"}
  ]
)
```

### 3. Validate
Call `wm_validate.check` to verify graph health.
