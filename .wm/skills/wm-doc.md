---
name: wm-doc
description: Create or update wiki pages
---

# Wiki Docs

**Announce:** "Using wm-doc."

## Steps

### 1. Search for existing
```
wm_search.query(q="<topic>", mode=keyword, limit=5)
```

### 2. Create or update
Create new:
```
wm_page.create(path="wiki/<type>/<slug>.md", title="<Title>", content="...")
```

Update existing:
```
wm_page.update(id="wiki:<type>:<slug>", content="...")
```

### 3. Add relationships
```
wm_page.update(
  id="wiki:<type>:<slug>",
  relates_to=[
    {type: extends, target: "wiki:concepts:<parent>"},
    {type: references, target: "wiki:reference:<source>"}
  ]
)
```

### 4. Verify
```
wm_validate.check()
```
