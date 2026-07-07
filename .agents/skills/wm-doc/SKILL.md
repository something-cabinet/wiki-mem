---
name: wm-doc
description: View, search, create, and update documentation
---

# Working with Docs

**Announce:** "Using wm-doc for [action]."

**Core principle:** STRUCTURED DOCS → AI-READABLE → CROSS-REFERENCED.

## Commands

```json
// List docs
wm_doc.list({})

// View doc
wm_doc.get({ "path": "<path>", "smart": true })

// Create doc
wm_doc.create({ "title": "<Title>", "folder": "<folder>", "content": "..." })

// Update doc
wm_doc.update({ "path": "<path>", "appendContent": "..." })

// Search docs
wm_search.query({ "query": "<topic>", "type": "doc" })
```

## Doc Types

| Folder | Purpose |
|--------|---------|
| `specs/` | Feature specifications |
| `patterns/` | Reusable design patterns |
| `decisions/` | Architecture decision records |
| `howto/` | Guides and tutorials |
| `reference/` | API and configuration reference |
| `concepts/` | Domain concepts and glossary |
| `learnings/` | Debugging patterns and discoveries |
