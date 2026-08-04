---
title: 'Failure: ChangeDetectionStrategy Eager vs Default confusion'
type: concept
id: wiki:concepts:angular-cd-eager-default-deprecation
relates_to:
  - {type: references, target: wiki:patterns:critical-patterns}
---
id: wiki:concepts:angular-cd-eager-default-deprecation

---
id: wiki:concepts:angular-cd-eager-default-deprecation
title: Failure: ChangeDetectionStrategy Eager vs Default confusion
type: concept
tags: [failure, angular, change-detection]
---
id: wiki:concepts:angular-cd-eager-default-deprecation

## What went wrong

A designer "fixed" all `ChangeDetectionStrategy.Eager` to `.Default` across 8 Angular components, believing `.Default` was the only correct value and `.Eager` didn't exist.

## Root cause

Angular deprecated `ChangeDetectionStrategy.Default` in favor of `.Eager` starting in Angular 22. Both enums have the value `1`, so the behavior is identical:
- `.Eager` is the **current recommended** value (the enum member that will remain).
- `.Default` is the **deprecated alias** scheduled for removal.

The designer assumed `.Eager` wasn't valid because they hadn't encountered it before, and didn't verify against the installed Angular version's types.

## Prevention

- When changing a value that compiles and works, verify the API against the installed package types before declaring it a "bug fix."
- For Angular ChangeDetectionStrategy: use `.Eager` in Angular 22+, not `.Default`.
- Check `node_modules/@angular/core` types before project-wide changes to unfamiliar APIs.

## Time lost

~30m Oracle review + 5m revert = ~35m total.

## Related
- @wiki/patterns:critical-patterns