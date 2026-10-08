---
title: Separate service ports over monolithic EnginePort
type: memory
tags: [angular, architecture, services, engineport]
status: active
---

schema_version: 1
state: |-
  Each distinct API domain gets its own port interface + InjectionToken + HTTP impl + mock impl. Avoids interface bloat, mock contamination, and allows independent evolution. Applied in CodeIntelPort. Full reference: @wiki/decisions/separate-service-ports-over-monolithic-engineport
questions:
  - id: layer
    type: choice
    instructions: Which memory layer does this entry belong to?
    options:
    - project
    - global
    - session
  - id: store_or_skip
    type: noul
    instructions: This entry is worth storing as durable memory.
  - id: dedup_action
    type: choice
    instructions: How should this entry relate to existing memory?
    options:
    - new
    - merge
    - supersede
    - skip
  - id: confidence
    type: score
    instructions: How confident is the recorded knowledge?
    levels:
    - low
    - medium
    - high
answers: {}
