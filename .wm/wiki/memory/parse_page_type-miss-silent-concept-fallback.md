---
title: parse_page_type miss — silent concept fallback
type: memory
tags: [failure, parser, enum]
status: active
---

When adding PageType::Core, missed parse_page_type() in apps/wm-core/src/parser/mod.rs. Caused silent concept fallback caught via graph stats. Use the 8 touch points checklist (@wiki/patterns/page-type-registration-touch-points) to prevent this.