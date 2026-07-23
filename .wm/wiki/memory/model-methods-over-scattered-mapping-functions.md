---
title: Model methods over scattered mapping functions
type: memory
tags: [decision, architecture, rust, enum]
status: active
---

Put enum to_str/from_str on the model, not in scattered standalone functions. Single match block eliminates drift, makes mapping discoverable via TypeName::. Full reference: @wiki/decisions/model-methods-over-scattered-mappings