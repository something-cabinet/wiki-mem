---
title: Add wm health audit CLI command skeleton
type: task
tags:
- from-spec
- spec:rebuild-log-findings
status: done
priority: high
acceptance_criteria:
  - text: "wm health audit CLI command exists with a HealthAction::Audit subcommand supporting --dry-run, --fix, and --format json|text flags"
  - text: "Audit command produces output formatting and a summary report"
---

Add Health variant to Commands enum with HealthAction::Audit subcommand supporting --dry-run, --fix, --format json|text flags. Includes output formatting and summary report.