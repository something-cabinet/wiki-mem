---
title: gh-ingest
description: Guided workflow for ingesting a raw source into the wiki.
trigger:
  event: source.complete
  priority: 1
---

## Steps

### Step 1: Review the source
Call `source.list(state="pending")` to see new sources.

### Step 2: Process and complete
Call `source.process(id="<id>")` then `source.complete(id="<id>")` to ingest.
