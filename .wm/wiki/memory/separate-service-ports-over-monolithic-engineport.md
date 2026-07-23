---
title: Separate service ports over monolithic EnginePort
type: memory
tags: [angular, architecture, services, engineport]
status: active
---

Each distinct API domain gets its own port interface + InjectionToken + HTTP impl + mock impl. Avoids interface bloat, mock contamination, and allows independent evolution. Applied in CodeIntelPort. Full reference: @wiki/decisions/separate-service-ports-over-monolithic-engineport