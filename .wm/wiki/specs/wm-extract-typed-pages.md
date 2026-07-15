---
title: wm-extract — Create Typed Pages, Not Just Learnings
type: spec
status: approved
tags: [extract, skills, wiki-types]
---

## Overview

The wm-extract skill currently produces only `wiki:learnings/` (concept type) and `wiki:patterns/` (pattern type) pages. It never creates `decisions`, `concepts`, `howto`, or `reference` pages. `wiki:learnings/` is eliminated — every finding goes directly into its proper typed page.

## PageType mappings

| Extraction finding type | Wiki directory | PageType | Frontmatter fields |
|---|---|---|---|
| Reusable code/arch pattern | `wiki:patterns/` | pattern | title, type, status, tags, confidence |
| Architecture decision (ADR) | `wiki:decisions/` | decision | title, type, status, tags, decision.context, decision.options, decision.rationale, decision.outcome |
| Domain concept | `wiki:concepts/` | concept | title, type, status, tags |
| Step-by-step guide | `wiki:howto/` | howto | title, type, status, tags |
| Reference / API doc | `wiki:reference/` | reference | title, type, status, tags |
| Failure / debugging story | `wiki:concepts/` | concept | title, type, status, tags |
| Raw informal notes | `wiki:concepts/` as note type | note (page_type: note) | title, type: note, status, tags |

## Requirements

### FR-1: Create separate typed pages per finding
Each extraction finding creates its own wiki page with the correct PageType and directory. No more lumping into a single "learnings" doc.

### FR-2: Decision pages get ADR frontmatter
```yaml
---
title: Use Wire for DI
type: decision
status: accepted
tags: [architecture, di]
decision:
  context: We need a DI framework
  options: [Wire, Dig, Manual]
  rationale: Compile-time safety
  outcome: Wire chosen
---
```

### FR-3: Update SKILL.md
Replace the extraction mapping table with the correct one above. Remove all references to `wiki:learnings/`.

### FR-4: Remove existing learnings dir content (opt-in)
The current `wiki:learnings/` directory stays as-is. New extractions go to proper types. A future cleanup can migrate existing entries manually.

### NFR-1: Backward compatible
Existing `wiki:learnings/` pages are not deleted.

## Acceptance Criteria
- [ ] AC-1: `wm-extract` creates decision pages in `wiki:decisions/`
- [ ] AC-2: `wm-extract` creates concept pages in `wiki:concepts/`
- [ ] AC-3: `wm-extract` creates howto pages in `wiki:howto/`
- [ ] AC-4: `wm-extract` creates reference pages in `wiki:reference/`
- [ ] AC-5: `wm-extract` creates pattern pages in `wiki:patterns/` (unchanged)
- [ ] AC-6: `wm-extract` no longer creates `wiki:learnings/` pages
- [ ] AC-7: SKILL.md updated with correct type table and no learnings references
