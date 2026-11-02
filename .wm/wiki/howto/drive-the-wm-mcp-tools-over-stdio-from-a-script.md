---
title: 'Howto: drive the wm MCP tools over stdio from a script'
type: howto
id: "wiki:howto:drive-the-wm-mcp-tools-over-stdio-from-a-script"
status: reviewed
tags: [howto, mcp, tool-reliability]
relates_to:
  - {type: relates_to, target: wiki:tasks:add-wm-cli-migrate-migrate-the-whole-wiki-with-versioned-records}
---

schema_version: 1
state: |-
  ## When

  The MCP server a client started is old or disconnected, and wm tools return success without writing (the 0.5.2 check_ac did) or Not connected. Start a fresh wm-cli mcp process for the call instead.

  ## Steps

  1. Write one JSON-RPC message per line to a file. The first two lines are always initialize and notifications/initialized. Each tool call is a tools/call message with a distinct id.

  ```json
  {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"manual","version":"0"}}}
  {"jsonrpc":"2.0","method":"notifications/initialized"}
  {"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"wm_task","arguments":{"action":"check_ac","id":"wiki:tasks:example","index":0}}}
  ```

  2. Pipe the file in and keep stdin open long enough for the replies. In fish:

  ```bash
  begin; cat calls.jsonl; sleep 8; end | wm-cli mcp 2>/dev/null | grep -E '"id":[2-9]'
  ```

  3. Read the written file back. A reply that says updated is not proof of a write.

  ## Gotchas

  - Do not send two writes to the same page in one file when the server may run them concurrently. 0.6.1 serialises read-modify-write handlers, but check the result anyway.
  - In 0.6.1, create accepts extra_frontmatter together with content. Spec and task pages are rejected without acceptance_criteria.
  - Generate the file with a script when the content has quotes or newlines. Hand-escaped JSON breaks easily.
  - Use the newest binary on PATH. Check wm-cli --version first.

  ## Verification

  grep the written markdown file for the field you changed, and run wm-cli validate.
questions:
  - id: task_kind
    type: choice
    instructions: What kind of task does this howto cover?
    options:
    - setup
    - development
    - testing
    - release
    - debugging
    - operations
    - integration
  - id: prerequisites_complete
    type: noul
    instructions: All prerequisites for this howto are fully documented.
  - id: difficulty
    type: score
    instructions: What skill level does this howto require?
    levels:
    - beginner
    - intermediate
    - advanced
    - expert
  - id: has_verification
    type: noul
    instructions: This howto includes a verification step.
answers:
  task_kind: operations
  prerequisites_complete: true
  difficulty: intermediate
  has_verification: true