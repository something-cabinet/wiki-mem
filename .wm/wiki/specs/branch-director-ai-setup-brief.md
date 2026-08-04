---
title: Branch Director AI Setup Brief
type: spec
status: approved
tags: [spec, team-ai, briefing, branch-director, wiki-mem, omo-slim, approved]
---

## Overview

A 1–2 page Markdown brief to send to the branch director describing the team's AI-assisted development setup built on **wiki-mem (wm)** — the local knowledge-graph memory layer — and **omo-slim (oh-my-opencode-slim)** — the OpenCode agent configuration plugin. The doc informs, demonstrates value with concrete metrics, and proposes rolling the setup out to the wider team.

## Locked Decisions

- D1: The spec describes the briefing document deliverable (not the setup implementation).
- D2: Mixed purpose — inform the director, show value, end with a modest ask.
- D3: Non-technical audience — plain language, outcomes over tooling, minimal jargon; any term like "MCP" or "CLI" must be explained in one phrase.
- D4: Concrete metrics are cited where available; qualitative wins fill gaps.
- D5: Markdown brief, 1–2 pages, scannable (short sections, bullets, one clear summary line).
- D6: Closing ask — propose rollout of the setup to more team members / other teams.

## Requirements

### Functional Requirements

- FR-1: The doc opens with a one-paragraph executive summary stating what the setup is and the single headline outcome.
- FR-2: The doc explains, in plain language, what each component does:
  - wiki-mem (wm): persistent project memory/knowledge base that agents and humans share (tasks, specs, decisions, searchable knowledge).
  - omo-slim: a plugin that configures the AI coding assistant (OpenCode) — agents, model choices, skills, and project conventions.
- FR-3: The doc includes a short "how the pieces fit together" section (1 short paragraph or a simple diagram) showing the daily workflow: AI assistant works on tasks → records decisions/knowledge in the wiki → future sessions reuse that memory.
- FR-4: The doc has a "Results / value so far" section citing concrete metrics (time saved, fewer bugs, faster turnarounds, docs quality) with 1-line evidence; qualitative examples fill any gaps.
- FR-5: The doc ends with a modest proposal: rollout to more team members (what it would take, why it's low-risk).
- FR-6: The doc is 1–2 pages, scannable: short paragraphs, bullets, bolded key numbers, no deep technical content.

### Non-Functional Requirements

- NFR-1: Plain language — a non-technical director can understand it without prior AI-tooling knowledge.
- NFR-2: Honest framing — metrics are accurate and attributable; no inflated claims.
- NFR-3: Standalone — readable without access to the repo or wiki.
- NFR-4: Portable format — Markdown that can be pasted into email or attached as-is.

## Acceptance Criteria

- [ ] AC-1: Doc is a single Markdown file, 1–2 pages when rendered.
- [ ] AC-2: Opens with an executive summary including the headline outcome.
- [ ] AC-3: Each component (wiki-mem, omo-slim) is explained in one plain-language paragraph or less.
- [ ] AC-4: Includes a "value so far" section with at least 3 concrete, believable metrics or evidence points.
- [ ] AC-5: Ends with a rollout proposal (what, why low-risk, what's needed).
- [ ] AC-6: Contains no unexplained technical jargon; any technical term is defined inline once.
- [ ] AC-7: Director-review test: a non-technical reader can summarize the doc in 2 sentences after one read.

## Scenarios

### Scenario 1: Happy Path
**Given** the director has no AI-tooling background
**When** they read the brief top to bottom
**Then** they understand what the team uses, why it works, what it has delivered, and what is being proposed — and can reply with a simple yes/ask follow-up.

### Scenario 2: Sceptical Director
**Given** the director questions whether AI assistance is reliable or secure
**When** they read the value and rollout sections
**Then** the doc shows attributable results and addresses how the setup keeps control (local memory, human review of commits, no third-party services for core function).

### Scenario 3: Metrics Gap
**Given** a specific metric is unavailable
**When** the section is written
**Then** the doc uses a believable qualitative example instead of fabricating a number, keeping the claims honest.

## Open Questions

- [ ] Exact metrics the user can cite (time saved, bug counts, turnaround examples) — user to supply before drafting the value section.
- [ ] Team/rollout specifics: team size, what onboarding support exists, who champions it.
- [ ] Whether the closing ask should name a target (e.g., "N team members by QX") or stay open.