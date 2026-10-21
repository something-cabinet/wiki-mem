---
title: CI Cargo audit fails on three zip advisories pulled by candle-core
type: task
id: "wiki:tasks:ci-cargo-audit-fails-on-three-zip-advisories-pulled-by-candle-core"
status: done
priority: high
tags: [ci, security, audit, dependencies]
acceptance_criteria:
  - text: "Cargo audit passes in CI with the three IDs ignored and a checked-in rationale"
  - text: "Rationale records that candle-core never calls the vulnerable extract APIs and the feature is off by default"
  - text: "A follow-up task tracks removing the ignores once candle-core allows zip >= 9"
  - text: "CI check job is green end to end"
relates_to:
  - {type: implements, target: wiki:specs:cargo-audit-zip-advisories}
implementation_notes: 'Committed as d2fb266. .cargo/audit.toml ignores the three ids with a rationale; cargo audit exits 0 locally and CI run 39401442935 is fully green. Follow-up: wiki:tasks/remove-zip-audit-ignores-when-candle-core-allows-zip--9.'
---

CI run 39400658945 now reaches `Cargo audit` and fails: 3 vulnerabilities found, all in `zip 8.6.0` — RUSTSEC-2026-0367 (SUID/SGID preservation on extract), RUSTSEC-2026-0368 (symlink escape on extract), RUSTSEC-2026-0369 (permissions applied through symlinks). Fixed in zip >= 9.0.0.

Dependency path: `zip` <- `candle-core 0.11.0` (`version = "8.6.0"`, caret) <- optional `decision` feature in `apps/wm-core` (off by default). candle-core uses `ZipArchive::new`, `by_name`, and `ZipWriter`; it never calls the vulnerable `ZipArchive::extract` / `ZipStreamReader::extract`. No newer candle-core exists (latest is 0.11.0), so zip 9 cannot be selected yet.

Direction: add `.cargo/audit.toml` ignores for the three IDs with a written rationale, plus a follow-up task to lift the ignores when candle-core allows zip >= 9.