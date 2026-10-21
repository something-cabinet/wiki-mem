---
title: Bump rustls-webpki and webpki roots for RUSTSEC-2026-0381/0382/0383
type: task
id: "wiki:tasks:bump-rustls-webpki-and-webpki-roots-for-rustsec-2026-038103820383"
status: todo
priority: high
tags: [security, dependencies, audit]
acceptance_criteria:
  - text: "Cargo.lock resolves rustls-webpki >= 0.103.16 and webpki-roots/webpki-root-certs >= 1.0.10"
  - text: "cargo audit exits 0"
  - text: "cargo check passes"
  - text: "CI check job is green"
relates_to:
  - {type: implements, target: wiki:specs:webpki-advisory-bump}
---

CI run 39633586288 failed Cargo audit on three newly disclosed advisories: rustls-webpki 0.103.15 (RUSTSEC-2026-0382, distrusted emSign intermediates), webpki-roots 1.0.8 (RUSTSEC-2026-0381), and webpki-root-certs 1.0.8 (RUSTSEC-2026-0383). Patched versions: rustls-webpki >= 0.103.16, webpki-roots/webpki-root-certs >= 1.0.10. Updated the lockfile with cargo update; no source changes.