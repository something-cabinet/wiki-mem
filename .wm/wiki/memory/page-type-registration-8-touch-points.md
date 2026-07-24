---
title: Page Type Registration — 8 touch points
type: memory
tags: [pattern, type-system, enum]
status: active
---

Adding a new PageType requires updating 8 locations: enum, page variant, parser/mod.rs parse_page_type, page/mod.rs filter+create, lint.rs, reference_service.rs, styles.css, test setup dirs. Missing parser/mod.rs causes silent concept fallback. Full reference: @wiki/patterns/page-type-registration-touch-points