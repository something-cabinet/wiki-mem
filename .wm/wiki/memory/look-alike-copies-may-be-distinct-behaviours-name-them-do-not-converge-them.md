---
title: Look-alike copies may be distinct behaviours - name them, do not converge them
type: memory
tags: [refactor, duplication, naming]
status: active
---

Three of five `slugify` copies had different semantics and their outputs become page ids and file paths, so converging them would have renamed existing pages. Diff copies on adversarial inputs (underscore, ampersand, consecutive spaces, empty), ask what each result is used for, name each behaviour honestly, and delete only byte-identical copies. Changing a behaviour that produces an identifier or path is a migration, not a tidy-up. Full reference: @wiki/patterns/name-distinct-behaviours-do-not-converge-them