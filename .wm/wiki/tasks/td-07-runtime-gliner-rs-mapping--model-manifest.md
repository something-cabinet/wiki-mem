---
title: 'TD-07 Runtime: gliner-rs mapping + model manifest'
type: task
id: "wiki:tasks:td-07-runtime-gliner-rs-mapping--model-manifest"
status: done
priority: medium
tags: [from-spec, spec:typed-decision-doc-format, runtime]
spec: specs/typed-decision-doc-format
acceptance_criteria:
  - text: "Record state/questions map into the model wire format"
  - text: "Runtime answers via gliner-rs offline"
  - text: "Model obtained via checksum-pinned manifest; not committed"
relates_to:
  - {type: relates_to, target: wiki:specs:typed-decision-doc-format}
---

Runtime integration: map record -> gliner-rs input, run choice/score/noul via its probability API, and add the model manifest + checksum-pinned download. Depends on TD-01.