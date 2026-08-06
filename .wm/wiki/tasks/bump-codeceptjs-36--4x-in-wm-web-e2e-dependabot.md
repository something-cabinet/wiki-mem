---
title: Bump codeceptjs 3.6 → 4.x in wm-web-e2e (dependabot)
type: task
id: wiki:tasks:bump-codeceptjs-36--4x-in-wm-web-e2e-dependabot
status: done
priority: medium
tags:
- security
- deps
- e2e
acceptance_criteria:
- text: codeceptjs upgraded to ^4.1.0 in apps/wm-web-e2e/package.json
- text: codecept.conf / helpers migrated for codeceptjs 4 API
- text: e2e suite passes headless (npm run test:e2e)
- text: npm audit in apps/wm-web-e2e shows 0 high-severity vulns
implementation_notes: |-
  Migration complete (codeceptjs 3.6 -> 4.1.0): ESM switch, tsx loader, export default page objects, locate import for noGlobals. check + dry-run pass; full suite 20/26 (6 pre-existing failures: Pages x3 no Create Page button in app + empty pages mock; Tasks x3 empty board mock). Remaining are test/mock-data gaps, not migration regressions.
  Gaps closed 2026-08-06: tasks-board/pages-list mocks populated with realistic data; added pages-get stub; rewrote 3 stale Pages journeys to real app behavior (no Create Page feature exists; badges assert text, not hlm-badge directive tag). Full suite now 26/26 passing headless.
---

All remaining wm-web-e2e dependabot alerts (axios, serialize-javascript, uuid, diff, joi, mocha — ~14 alerts) resolve only via codeceptjs 4.1.0 major bump. Requires e2e config migration (codecept.conf / step definitions).

Related: codeceptjs 4 is a framework major; verify codecept.conf.js and custom helpers before switching.

From dependabot sweep 2026-08-06: safe fixes done (undici, fast-uri); this is the remaining major.