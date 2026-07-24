---
title: Add PageType::Core enum variant and Page::Core enum variant
type: task
tags:
- from-spec
- spec:core-page-type
status: done
priority: high
acceptance_criteria:
- text: PageType::Core compiles with as_str() returning 'core'
  checked: false
- text: Page::Core { meta } variant compiles with From impls
  checked: false
- text: priority_rank returns 9 for Core
  checked: false
- text: allowed_statuses returns [Draft, Reviewed, Approved, Archived]
  checked: false
- text: Tests added to lib.rs for Core
  checked: false
implementation_notes: 'Extracted pattern: @wiki/patterns/page-type-registration-touch-points — documents the 8 touch points for adding PageTypes'
---

Add Core variant to PageType enum (as_str, allowed_statuses, priority_rank) and Core { meta } variant to Page enum with From impls. Write tests.