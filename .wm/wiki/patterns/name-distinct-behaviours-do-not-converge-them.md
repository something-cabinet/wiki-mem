---
title: 'Pattern: Name distinct behaviours, do not converge them'
type: pattern
id: "wiki:patterns:name-distinct-behaviours-do-not-converge-them"
status: reviewed
tags: [pattern, refactor, duplication, naming, migration]
relates_to:
  - {type: references, target: wiki:specs:wm-core-layer-extraction}
---

schema_version: 1
state: |-
  ## Problem

  A duplication hunt finds five copies of `slugify`. Consolidating them looks like free cleanup. It is not: three of the five have **different semantics**, and their outputs become **page ids and file paths** - so picking a winner silently renames existing pages.

  Measured on the three implementations:

  | input | variant A | variant B | variant C |
  |---|---|---|---|
  | `my_task` | `my-task` | `my_task` | `mytask` |
  | `Auth & Security!` | `auth-security` | `auth-security` | `auth--security` |
  | `Hello  World` | `hello-world` | `hello-world` | `hello--world` |

  They are not duplicates. They are **three operations wearing one name**, and the name is the defect.

  ## Solution

  Before converging any set of look-alike copies:

  1. **Diff them on adversarial inputs**, not typical ones - underscore, ampersand, punctuation, consecutive whitespace, leading/trailing space, empty string, non-ASCII. Typical inputs hide the divergence.
  2. If the outputs differ, **do not pick a winner by taste.** Ask what each call site's result is *used for*: an identifier, a filesystem path, a URL segment, or a human-facing label. Different uses justify different rules.
  3. **Name each behaviour for what it does**, including where the result becomes an identifier or a path. The name is the deliverable.
  4. Keep one implementation per behaviour. Delete only the **byte-identical** copies.
  5. Pin each behaviour with a test built from the adversarial inputs above, so a future consolidation cannot rename a page unnoticed.

  ## When to Use

  Any "N copies of the same helper" finding, and especially any helper whose output reaches an identifier, a path, or a persisted key.

  ## When Not to Use

  When the copies are byte-identical (converge freely), or when a behavioural difference is provably unobservable at every call site. Note that "observable" includes *on disk*, not just in the current process.

  ## The decision this defers

  Changing a behaviour that produces an identifier or a path is a **migration**, not a tidy-up: existing records carry the old value. That belongs to the owner, not to whoever is refactoring - which is exactly why the right move here was to name the behaviours and leave the values alone.

  ## Related

  - @wiki/specs:wm-core-layer-extraction
  - @wiki/reference:wm-core-architecture-inventory
questions:
  - id: problem_kind
    type: choice
    instructions: What kind of problem does this pattern solve?
    options:
    - architecture
    - api-design
    - data-model
    - error-handling
    - performance
    - testing
    - ui
    - tooling
    - workflow
  - id: preconditions_required
    type: noul
    instructions: This pattern requires specific preconditions to be met.
  - id: complexity
    type: score
    instructions: How complex is applying this pattern?
    levels:
    - trivial
    - simple
    - moderate
    - complex
  - id: language_specific
    type: noul
    instructions: This pattern is specific to a programming language.
answers: {}