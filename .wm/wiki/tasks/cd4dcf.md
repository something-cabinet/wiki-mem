---
title: "Audit: investigate Angular bundle size increase"
id: cd4dcf
type: task
status: done
tags: [review, frontend, build, performance]
priority: medium
acceptance_criteria:
  - text: "ng build --stats-json produces a bundle analysis identifying the top contributors to the size increase (spartan-ng hlm-sidebar, @ng-icons/core icon packs, fjadra layout)"
  - text: "Mitigation options evaluated: tree-shaking spartan-ng imports, lazy loading NgIcon providers per view, code-splitting fjadra-related code"
  - text: "Budgets restored to tighter limits (maximumWarning 500kB / maximumError 1MB) if feasible, or a justification recorded for keeping the raised limits"
---

# Audit: investigate Angular bundle size increase

## Description

The Angular budget in `angular.json` was increased from `maximumWarning: 500kB → 1MB` and `maximumError: 1MB → 2MB`, suggesting bundle size doubled. Likely contributors: spartan-ng `hlm-sidebar` components, `@ng-icons/core` with multiple icon packs, and the fjadra layout code.

## Location

`apps/wm-web/angular.json` — budgets section

## Acceptance Criteria

- [ ] Run `ng build --stats-json` and analyze bundle composition
- [ ] Identify top contributors to size increase
- [ ] Consider:
  - Tree-shaking spartan-ng imports (only import used components)
  - Lazy loading NgIcon providers per view instead of globally
  - Code-splitting fjadra-related code
- [ ] Optionally restore tighter budgets if feasible
