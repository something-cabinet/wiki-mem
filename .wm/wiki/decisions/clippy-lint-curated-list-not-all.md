---
title: 'Decision: Curated Clippy Lint List + clippy.toml Over All = Warn'
id: wiki:decisions:clippy-lint-curated-list-not-all
type: decision
tags:
- decision
- clippy
- lint
- code-quality
status: approved
relates_to:
  - {type: references, target: wiki:specs:wm-doc-type-frontmatter}
---

---
title: 'Decision: Curated Clippy Lint List + clippy.toml Over All = Warn'
id: wiki:decisions:clippy-lint-curated-list-not-all
type: decision
relates_to:
  - {type: references, target: wiki:specs:clippy-lint-cleanup}
status: approved
tags: [decision, clippy, lint, code-quality]
---

## Context

The project needed to select a clippy lint configuration approach. Two models were considered: egui's approach (enable `all = warn`, then selectively allow) and a curated list of specific lints. A campaign to fix ~400 lint warnings demonstrated that `as_conversions` and `cast_possible_truncation` from the `restriction` group produce false positives that lead to worse code (25 lines of bit-twiddling replacing `score as f32`).

## Decision

Use a **curated list** of specific clippy lints in `[workspace.lints.clippy]` plus a `clippy.toml` file for configuration. Do NOT use `all = { level = "warn", priority = -1 }`.

## Rationale

- **Restriction-group lints (`as_conversions`, `arithmetic_side_effects`) fire on correct code.** The vector_db bit-twiddle demonstrated the worst case: 25 lines of fragile bit manipulation, 3 magic numbers, 6 comments, and a NaN-masking bug — all to dodge a one-line `as f32` cast.
- **Test code needs different treatment.** `allow-unwrap-in-tests = true` in `clippy.toml` makes `unwrap_used` viable for production code.
- **Agent fixers optimize for "make the lint stop," not "make the code better."** A broad lint set causes churn without improvement.
- **egui's approach only works for them because they have maintainer bandwidth** to maintain an allow-list and manual cherry-pick from new lints at toolchain bumps.

## `#[allow]` Policy

Named, item-scoped `#[allow(clippy::lint_name, reason = "...")]` is permitted when:
- The code is correct and the lint cannot see why
- The reason states the invariant ("score is cosine distance in [0,2]; f32 precision is sufficient")

`#[allow]` is forbidden when:
- Hiding defects (dead code, unused imports, wildcard `clippy::*`)
- The fix is mechanical (use `From` instead of `as`, use `let-else` instead of `unwrap_or`)

Prefer `#[expect]` over `#[allow]` — warns when the lint stops firing, so suppressions self-clean.

Code-contortion to silence a lint is a defect. If the choice is between a one-line reasoned `#[allow]` and a workaround that is longer, subtler, or less safe, the `#[allow]` wins.

### 2026-08-14 Amendment: `#[allow(dead_code)]` is banned entirely

Issue #126 (wm_doc drops `type` frontmatter) was caused by a **dead input field masked by `#[allow(dead_code)]`**: `WmDocAction::Create.r#type` was declared in the schema but never wired in the handler, and the annotation silenced the compiler warning that would have caught it. Verdict: `#[allow(dead_code)]` on any API input field is a contract violation — the annotation's only function was hiding a defect.

Enforcement (verified — no clippy lint can ban attributes):
- `clippy::disallowed_attrs` does not exist; `clippy::allow_attributes_without_reason` only forces reasons, it cannot ban.
- Therefore a **deterministic CI grep** in the `check` job bans `#[allow(dead_code)]` repo-wide (`#\[allow\(\s*dead_code[,\)]`, excluding `target/`, `node_modules/`, `.wm/`).
- The sanctioned replacement is `#[expect(dead_code, reason = "...")]` — it errors when the lint does NOT fire, so it self-removes once the item is used and can never mask a live field (rustc 1.81+; the repo has no MSRV pin, CI uses latest stable).

## Consequences

- New clippy lints must be evaluated manually at toolchain bumps
- `clippy.toml` is the canonical place for lint configuration (thresholds, allow lists, exclusions)
- Workspace `Cargo.toml` `[workspace.lints.clippy]` for lint levels only
- `#[allow(dead_code)]` never appears in the repo; CI enforces it (spec wm-doc-type-frontmatter D3/FR-6)

## Related
- @wiki/specs:clippy-lint-cleanup
- @wiki/rules/no-warnings
- @task-deadcode-ban-enforcement (2026-08-14 ban execution)
- @wiki/specs/wm-doc-type-frontmatter (issue #126 root cause + ban spec)