---
title: Upgrade weak references to stronger types in wiki graph
id: 0ddcd1
type: task
status: done
priority: low
tags:
- graph
- edges
- quality
acceptance_criteria:
- text: 'Weak references edges are upgraded to stronger typed edges where semantically valid: concept→concept to extends, task→spec to implements, spec→decision to answers, pattern→concept to example_of/extends'
- text: Upgrades start with concept→concept and concept→spec connections where one clearly extends/specializes the other, per the priority heuristic
- text: The graph reflects the upgraded edge types (verified via graph rebuild / edge audit)
---

After the edge-verification deepwork session, ~207 edges are `references` (priority 1). Many could be upgraded to stronger typed edges that tooling can act on. 

Target opportunities:
- Concept→Concept: `references` → `extends` (when one is a specialization of the other)
- Task→Spec: `references` → `implements` (when task implements the spec)
- Spec→Decision: `references` → `answers` (when decision answers spec question)
- Pattern→Concept: `references` → `example_of` or `extends`
- Memory→Source: `references` → `references` is fine (memory is always a reference)

The tag-cluster analysis identified these upgrade candidates:
- `graph` tag cluster: concepts/edge-types → specs/graph-engine could be `extends` not `references`
- `search` tag cluster: concepts/bm25-search → patterns/field-weighted-bm25 could be `example_of`
- `failure` tag cluster: all failure concepts could share `references` but some may be `extends`
- `migration` tag cluster: specs/sim-ui-full-migration → tasks could be `implements`

Priority heuristic: start with concept→concept and concept→spec connections where one clearly extends/specializes the other.