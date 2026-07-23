---
title: Fix PostCSS config for Angular 22
type: task
status: done
relates_to:
  - {type: references, target: wiki:notes:session-handover-2026-07-17}
---

\postcss.config.json\ (not .js!) is required for Angular 22's @angular/build to pick up Tailwind v4. The .js format is silently ignored.

Action: Create or rename postcss.config.json in apps/wm-web.

References: @wiki/notes/session-handover-2026-07-17.md