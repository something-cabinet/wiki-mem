---
title: Two-layer regression guards: lint + integration tests
type: memory
tags: [decision, testing, lint]
status: active
---

Use both lint checks (demand-driven, catches existing issues) AND integration tests (CI-driven, catches new regressions) for wiki health properties. Each layer has different trigger conditions and coverage. Full entry: @wiki/decisions/lint-plus-integration-tests-for-wiki-health