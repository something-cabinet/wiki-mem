---
title: system-one-02 Map WM workflows to decision framing + baselines
type: task
id: "wiki:tasks:system-one-02-map-wm-workflows-to-decision-framing--baselines"
status: done
priority: high
tags: [from-spec, spec:system-one-model-application, research, codebase]
spec: specs/system-one-model-application
acceptance_criteria:
  - text: "FR-3: each of the five workflows has its state, typed questions, and answer space described"
  - text: "FR-5 baselines: current heuristic/behaviour and metrics identified with file:line refs"
  - text: "Labelled/available data for training or evaluation identified per workflow"
  - text: "Read-only: no code changes"
relates_to:
  - {type: relates_to, target: wiki:specs:system-one-model-application}
---

schema_version: 1
state: |-
  Recon the Wiki-Mem codebase and map its five workflows (search relevance & routing, page & task classification, memory & rule decisions, MCP tool routing & review gates, graph edge typing) into decision-model framing: state, questions, answer space, current baseline, and available labelled data. Cover FR-3 and FR-5 baselines. Read-only.
questions:
  - id: work_kind
    type: choice
    instructions: What kind of work is this task?
    options:
    - feature
    - bugfix
    - refactor
    - docs
    - test
    - chore
    - migration
  - id: priority
    type: choice
    instructions: What priority is this task?
    options:
    - low
    - medium
    - high
    - urgent
  - id: needs_spec
    type: noul
    instructions: This task depends on a spec.
  - id: has_ac
    type: noul
    instructions: This task has at least one acceptance criterion.
answers: {}
