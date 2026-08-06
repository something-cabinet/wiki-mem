---
title: wm_search.retrieve for memory + status per-type + vectors.bin
type: task
status: done
tags: [from-spec, go-mode]
priority: high
id: fkx6g9
acceptance_criteria:
  - text: "wm_search.retrieve accepts a type param and assembles memory context as flat text with a 70/30 token budget split"
  - text: "wm_index.status reports per-type document counts"
  - text: 'vectors.bin is extended with the WMV\1 type tag format (backward compatible with WMV\0)'
---

# wm_search.retrieve for memory + status per-type + vectors.bin

> *Imported from Knowns task `fkx6g9`*

# wm_search.retrieve for memory + status per-type + vectors.bin

## Description


Extend retrieve with type param, flat text context for memory, token budget split 70/30. Extend wm_index.status per-type counts. Extend vectors.bin with WMV\1 type tag format.


## Acceptance Criteria
