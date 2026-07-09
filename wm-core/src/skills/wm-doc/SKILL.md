---
name: wm-doc
description: View, search, create, and update wiki documentation
---

# Working with Docs

**Announce:** "Using wm-doc for [action]."

**Core principle:** STRUCTURED DOCS → AI-READABLE → CROSS-REFERENCED.

## Inputs

- Action: list, get, create, update, or delete
- Page ID, content, folder, tags as needed

## Commands

### List All Docs

```json
wm_doc_list({})
```

### View a Page

```json
wm_page_get({ "id": "<page-id>", "smart": true })
wm_page_get({ "id": "<page-id>", "toc": true })
wm_page_get({ "id": "<page-id>", "section": "<heading>" })
```

### Create a Page

```json
wm_page_create({
  "id": "<folder>/<page-name>",
  "title": "<Page Title>",
  "tags": ["<search-keyword>"],  # Use specific search keywords (e.g., "api", "authentication"), not metadata
  "content": "..."
})
```

### Update a Page

```json
wm_page_update({ "id": "<page-id>", "appendContent": "..." })
wm_page_update({ "id": "<page-id>", "tags": ["updated", "tag"] })
```

### Delete a Page

```json
wm_page_delete({ "id": "<page-id>" })
```

### Search Docs

```json
wm_search_query({ "query": "<topic>", "type": "page" })
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

## Doc Writing Guidelines

- Use descriptive titles that work as search targets
- Cross-reference related pages using `@page/<page-id>` syntax
- Include a brief overview at the top of every page
- Use consistent heading structure (## sections, ### subsections)
- Keep pages focused on one topic — split large pages
- Tag pages appropriately for filtering

## Checklist

- [ ] Correct action selected (list/get/create/update/delete)
- [ ] Page IDs use consistent path structure
- [ ] Content follows wiki conventions
- [ ] Cross-references use `@page/` syntax
- [ ] Tags applied for discoverability
- [ ] Index rebuilt after create/update/delete

## Red Flags

- Creating pages with duplicate or conflicting IDs
- Writing long-form content without headings or structure
- Forgetting to cross-reference related pages
- Not tagging pages — they won't be discoverable by tag search
- Updating pages without reviewing existing content first

## Next Step Suggestion

```
/wm-spec              — Create a new spec document
/wm-extract           — Extract patterns into docs
/wm-commit            — Commit doc changes
```
