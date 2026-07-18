---
title: "FR-1: Extract schema structs from flat tool files"
type: task
status: todo
tags: [uniform-mcp-schema-structs, refactor]
task_data:
  acceptance_criteria:
    - text: "time.rs WmTimeAction variants have schema structs (Stop, Add, Report)"
      checked: false
    - text: "memory.rs WmMemoryAction variants have schema structs (Add)"
      checked: false
    - text: "index.rs WmIndexAction variants have schema structs (Embed)"
      checked: false
    - text: "doc.rs WmDocAction variants have schema structs (List)"
      checked: false
    - text: "graph.rs WmGraphAction variants have schema structs (Neighbors)"
      checked: false
    - text: "All #[allow(dead_code)] removed from these files"
      checked: false
  estimate: 2
  difficulty: "low"
---
