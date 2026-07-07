---
title: gh-implement
description: Implement changes per plan — code, docs, wiki updates.
---

## Steps

### Step 1: Review the plan
Call `wm_page.get(id="<task-id>")` to read the plan, prerequisites, and acceptance criteria.

### Step 2: Gather context
Search the wiki for related patterns, concepts, and reference pages using `wm_search.query` and `wm_search.retrieve`.

### Step 3: Implement changes
Update wiki pages, write code, add documentation. Link new pages to existing ones using `wm_page.link`.

### Step 4: Update task status
Call `wm_page.update(id="<task-id>")` with status and links to created pages.
