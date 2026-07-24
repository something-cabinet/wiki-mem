---
title: "Audit: investigate Angular bundle size increase"
id: cd4dcf
type: task
status: done
tags: [review, frontend, build, performance]
priority: medium
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
