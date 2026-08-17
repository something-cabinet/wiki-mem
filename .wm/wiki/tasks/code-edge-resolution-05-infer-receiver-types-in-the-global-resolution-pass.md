---
title: code-edge-resolution-05 Infer receiver types in the global resolution pass
type: task
id: "wiki:tasks:code-edge-resolution-05-infer-receiver-types-in-the-global-resolution-pass"
status: todo
priority: high
tags: [from-spec, spec:code-edge-resolution, p2, code-intel, resolution]
spec: wiki:specs:code-edge-resolution
acceptance_criteria:
  - text: "A Rust fixture where let x = Foo::new() then x.method() resolves method to Foo's defining file, not to every symbol named method (spec AC-2.2)"
  - text: "Receiver types are inferred from the enclosing impl or class for self and this method calls"
  - text: "Receiver types are inferred from declared bindings and from constructor calls in both Rust and TypeScript"
  - text: "Receiver types are inferred from typed function parameters"
  - text: "Cross-file return types are inferred via the symbol index so a binding from a function call resolves"
  - text: "Inference is deterministic — identical inputs produce byte-identical edge sets (spec NFR-2.1)"
relates_to:
  - {type: implements, target: wiki:specs:code-edge-resolution}
---

Phase 2 of wiki:specs:code-edge-resolution. Implements FR-2.3.

Adopted from Graphify's resolution.py per decision D5 — receiver-type inference is the half that makes member calls resolvable rather than ambiguous. Without it, capturing member calls in task 04 would emit mostly multi-candidate guesses, which under D3 get dropped, so 04 alone would deliver little.

Key insight from wiki:reference:graphify-adoption-assessment — receiver typing cannot be done purely file-locally. A binding like let x = make_thing() needs make_thing's return type from another file, so inference belongs in the single global resolution pass, not in extraction. Extraction supplies the receiver expression, this task resolves it to a type.

Inference sources, in order of confidence — enclosing impl or class for self and this, declared bindings such as let x with a type annotation, constructor calls such as Type::new() or new Type(), typed function parameters, and cross-file return types via the symbol index.

Constraint per NFR-2.1 — deterministic and local. No LLM, no network, no language-server subprocess. packages/wm-lsp resolves types exactly and is a tempting shortcut, but it is excluded here and tracked as an open question in the spec for a possible later opt-in verification pass.

Files: packages/wm-code-intel/src/services/graph_resolver.rs.