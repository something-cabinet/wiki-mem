---
title: 'Pattern: Order a guard after the fix it depends on'
type: pattern
id: "wiki:patterns:order-guards-after-their-enabling-fix"
status: reviewed
tags: [pattern, process, validation, ordering]
relates_to:
  - {type: references, target: wiki:tasks:wiki-tool-reliability-wmpageupdate-ignores-extrafrontmatter-with-content}
---

schema_version: 1
state: |-
  ## Problem

  You add a guard that rejects a bad state - a schema check, a lint, a CI gate, an assertion. But a **separate, already-proven defect** prevents callers from producing the good state. The new guard therefore rejects legitimate work, and a silent failure becomes a **loud wrong one**.

  The wrong version is worse than the silence it replaced. A silent failure leaves a bad artifact that someone may eventually notice; a guard that rejects a caller who did everything right teaches them the guard is broken, and they route around it.

  ## Solution

  Before shipping a guard, ask one question: **can a caller produce a conforming artifact today, through the paths this guard will judge?**

  If a known defect blocks that, the guard and the fix are coupled:

  1. Fix the enabling defect first, or
  2. Ship both together, and
  3. Record the dependency as a graph edge (`depends_on`) so the order is visible to whoever implements next - not buried in prose.

  ## The case that produced this

  `wm_validate.check` flags tasks with no acceptance criteria, and a task exists to enforce ACs at creation time. But `extra_frontmatter` is **silently discarded whenever `content` is supplied in the same call** (reproduced 2 for 2). A caller can therefore pass `acceptance_criteria` at creation today and receive an AC-less page with a success response.

  Enforce first, and that exact caller is rejected for a task whose ACs they *did* supply. The two fixes are recorded as `depends_on` from the enforcement task to the discard task.

  ## When to Use

  Whenever a guard is added to a system carrying a known open defect in the path that guard will judge - especially when the guard is aimed at cleaning up evidence the defect produced.

  ## When Not to Use

  When the judged path has no known blocking defect. Adding the guard early is then exactly right: that is how the defect surfaces in the first place. This pattern is about *ordering*, not about withholding guards.

  ## Related

  - @wiki/tasks:per-type-schema-enforcement--requiredoptional-typed-frontmatter-keys--mandatory-acs-for-task-and-spec
  - @wiki/tasks:wiki-tool-reliability-wmpageupdate-ignores-extrafrontmatter-with-content
  - @wiki/concepts:failure-guard-covered-only-one-representation
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