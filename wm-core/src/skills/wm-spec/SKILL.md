---
name: wm-spec
description: Create specification documents (Spec-Driven Development)
---

# Creating a Spec Document

**Announce:** "Using wm-spec to create spec for [name]."

**Core principle:** EXPLORE DECISIONS → SPEC → REVIEW → APPROVE.

## Phase 0: Exploring

Assess scope (quick/standard/deep). Identify gray areas. Ask one question at a time. Lock decisions with IDs.

## Step 1: Create Spec

```json
wm_doc.create({ "title": "<Feature Name>",
  "folder": "specs",
  "tags": ["spec", "draft"],
  "content": "<spec content>" })
```

Spec template: Overview, Locked Decisions, Requirements, Acceptance Criteria, Scenarios, Open Questions.

## Step 2: Validate

```json
wm_validate.check({ "entity": "specs/<name>" })
```

## Step 3: Review

Present for user review. On approval, update tags:

```json
wm_doc.update({ "path": "specs/<name>", "tags": ["spec", "approved"] })
```
