---
title: Add wm health audit CLI command skeleton
type: task
tags:
- from-spec
- spec:rebuild-log-findings
status: done
priority: high
---

Add Health variant to Commands enum with HealthAction::Audit subcommand supporting --dry-run, --fix, --format json|text flags. Includes output formatting and summary report.