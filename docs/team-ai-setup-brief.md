# Team AI Setup — Briefing

**For:** Branch Director
**From:** [Your Name]
**Date:** [Date]
**Status:** Informational + proposal

---

## Executive Summary

Our team runs an AI-assisted development setup built on a persistent project knowledge base (wiki-mem) and **Kiro** — an AI coding assistant configured to work as a team of specialised agents. Based on early, informal use — not yet formally measured — it appears to have cut hours spent on development by around **50%**, reduced rework by up to **80%**, and cut AI cost/usage by over **50%** — while keeping the compounding knowledge base itself (specs, decisions, task history) on our own machines rather than a third-party service (see §2 for the one exception: session context still goes to the model provider for inference, same as any AI coding assistant). We propose a tracked pilot (see below) to confirm these figures — currently based on a single team's early experience — before any wider rollout.

The vision is a development team where people set direction and review outcomes, while AI handles the heavy lifting of code, research, and documentation — with the knowledge base compounding our advantage on every project.

---

## 1. Agents — The Bone Structure

Think of the AI side of the setup as a small team of specialised assistants, coordinated by a lead. Each one runs on a specific model chosen to match the difficulty and risk of its job — harder, higher-stakes work gets a stronger (and more expensive) model; fast, well-defined, low-risk work gets a lighter, cheaper one. This is a deliberate cost/quality trade-off, not a default.

### Lead (Orchestrator)

**Model:** Claude Sonnet 5
**Job:** Plans the work, splits it into lanes, dispatches tasks to the right specialist, then reconciles and verifies the results.
**Why this model is sufficient:** The orchestrator's job is coordination, not deep problem-solving — it needs to read requirements, route work correctly, and check results, all of which a strong mid-tier model handles reliably. Reserving the top-tier model for the orchestrator would raise cost on every single task, including trivial ones, without improving routing quality.

### Explorer

**Model:** GPT-5.6 Luna
**Job:** Fast codebase searches — finds files, symbols, and patterns across the repo.
**Why this model is sufficient:** This is a mechanical retrieval task (search, grep, symbol lookup) rather than a reasoning task. A fast, lighter-weight model returns results quicker and cheaper, and search accuracy here depends more on good tooling (ripgrep, ast-grep, LSP) than on model depth.

### Librarian

**Model:** GPT-5.6 Terra
**Job:** Looks up current documentation and best practices from official docs and the web.
**Why this model is sufficient:** The task is retrieval-and-summarise, not novel reasoning — the model's job is to read external sources faithfully and report back, which doesn't require top-tier reasoning depth.

### Oracle

**Model:** Claude Opus 5
**Job:** Senior advice on architecture, tricky bugs, and code review.
**Why this model is sufficient:** Oracle is only invoked for high-risk decisions, unclear root causes, or costly trade-offs — situations where a wrong call is expensive, so paying for the strongest available reasoning is worth it. (A small number of other roles not detailed here — used for multi-model consensus on especially high-stakes calls — also run on this tier; it's reserved for stakes, not unique to one role.) It's deliberately not the default for routine work.

### Designer

**Model:** GPT-5.6 Terra
**Job:** UI/UX design, layout, responsive behaviour, and visual polish.
**Why this model is sufficient:** Design quality here comes largely from applied visual/interaction judgment and following design-system conventions already in the codebase, which this model handles well without needing the top reasoning tier reserved for architectural decisions.

### Fixer

**Model:** GPT-5.6 Luna
**Job:** Fast implementation of well-defined, bounded changes and tests.
**Why this model is sufficient:** Fixer is only handed clear, scoped instructions — no research or architectural decisions. For well-specified execution work, a fast model keeps turnaround quick and cost low without sacrificing correctness, since ambiguity has already been resolved before the task reaches Fixer.

### Kiro (the AI coding assistant)

Kiro is configured to run as this coordinated team of specialised agents, each with one job, guided by shared project rules — rather than as a single general-purpose assistant handling everything at one cost/quality level.

### Agents Are Stateless — By Design, Not By Limitation

Every LLM request is stateless by nature: an agent's model has no memory of its own between calls, so the full relevant context has to be sent with every request — the bigger that context, the more it costs. Instead of an agent building up a bigger and bigger conversation to remember prior work, it queries wiki-mem (covered next) for exactly what it needs — relevant docs, past decisions, task state — at the start of a session or task. Small, targeted reads replace a large, ever-growing history, so each agent request stays lean and cheap while the knowledge behind it keeps compounding.

---

## 2. wiki-mem — The Memory

wiki-mem is a persistent, searchable knowledge base that both people and AI agents share — the shared memory every agent reads from and writes to, regardless of which model is behind it. Two prior architecture decisions create problems that wiki-mem exists to solve: running **Spec-Driven Development (SDD)** — the spec → plan → implement → verify pipeline in Workflow, below — and running a **team of coordinated, specialised agents** (Agents, above) instead of one assistant. Both are good decisions on their own, but neither works well without memory behind it — so wiki-mem gives the team shared memory and the tools to use it. Everything below runs locally.

### Local-first data

wiki-mem's data — specs, tasks, decisions, memory — lives on our own machines, not a third-party service, and that's also why token usage is lower: agents start with context already loaded from local memory instead of re-reading or re-generating it each session — this is where the **over 50% reduction in AI cost/token usage** comes from (early, informal figure). The one exception is model inference itself — each agent still sends its session context to its model provider (Claude/GPT) to get a response, the same as any AI coding assistant. What stays local is the compounding knowledge base and the search that ranks it; what leaves the machine is the same prompt/code context any LLM call requires.

### How wiki-mem finds things

Search isn't a single mechanism — it's a layered stack that finds what you mean, not just what you type:

-   **Keyword search (BM25) with stemming** — reduces words to a common root before matching, so tense, plural, and other close grammatical variants of the same word are treated as the same term ("implement", "implementing", "implemented"; "design pattern" vs "design patterns"). It's fast, precise, and weights title matches far above body text. On its own it can't bridge the gap where the words are simply different (login vs. authentication).
-   **Semantic search (local ONNX embeddings)** — converts text to a numeric representation of _meaning_, not just words, so "login" and "authentication" land close together — entirely on our own machine, no external embedding API call needed.
-   **Rank fusion** — combines keyword and semantic results instead of picking one or the other, so a query matches on both exact wording and underlying concept.
-   **Reranking** — after fusion, a set of precision boosts reorders results so the best match rises to the top:
    -   _Exact title match_ — a page whose title contains the query words is boosted
    -   _Exact ID match_ — a page whose ID matches the query is boosted
    -   _Title density_ — the more query words in the title, the higher the boost
    -   _Tag overlap_ — pages sharing tags with the query rank higher
    -   _Recency boost_ — recently created or actively re-referenced entries rank higher, so current guidance outranks stale material
-   **A relational graph** — pages declare relationships to other pages (9 edge types — e.g. `references`, `implements`, `supersedes`, `depends_on` — not just a generic link) as part of normal work: the Spec step links a decision to the requirement it answers, the Extract step links a captured pattern back to its source task. Once those edges exist, traversing them costs nothing further — **no token cost and no LLM call**: graph traversal is pure local computation. So a deep-research or debugging session gets the whole neighbourhood around a page — the spec that decision implements, the pattern that superseded it, the task that hit the same failure before — without guessing search terms. The agent spends its tokens on reasoning, not on retrieval.

### Code intelligence

The wiki also understands the code itself. Its **code intelligence** layer parses the codebase into an AST (abstract syntax tree) and indexes symbols, definitions, and dependencies, so structural questions — "where is the login handler defined", "what does this function call", "who depends on this service" — are answered directly: symbol lookup, reference search, and dependency analysis return precise answers **without token cost and without an LLM call**, the same pure-local-computation pattern as the graph above. Agents spend their tokens on reasoning about the code, not on hunting through it.

### Problems

-   **A team of stateless agents would mean bloated, expensive context** — a multi-agent team means many separate LLM calls, each stateless with no memory between requests. Left unaddressed, you either resend a growing conversation transcript every time (expensive, and eventually hits a hard context limit) or start from zero each session (loses all prior project knowledge).

    -   **Solution:** wiki-mem acts as external memory each agent queries on demand — instead of carrying history forward, an agent asks a targeted question ("what does this project's Conventions doc say", "was this decided already") and gets back only the relevant slice. Context stays small; nothing is lost.

-   **SDD stacks specs, burying the current guidance** — running SDD means every task produces its own spec, locked decisions, and patterns, each adding a layer of guidance on top of the previous one. Left unmanaged, an agent can't tell current guidance from superseded guidance and may trust the stale one.

    -   **Solution:** the **recency boost** (see How wiki-mem finds things, above) ranks recent or actively re-referenced entries higher, so the current spec for a topic surfaces ahead of the stacked older ones — combined with the Extract step's explicit **dedup-and-update-over-duplicate rule** (Workflow, below), which merges or supersedes outdated pages instead of letting them pile up.

-   **Every new session normally starts from zero** — without persistent memory, an engineer (or an agent) re-explains the project, re-reads the same files, and re-discovers the same conventions every single time work starts.

    -   **Solution:** wiki-mem's Init step (see Workflow, below) loads that context automatically instead of rebuilding it by hand — this is where the **~50% reduction in development hours** comes from (early, informal figure).

-   **Requirements get assumed during coding, then corrected after the fact** — without an explicit decision-locking step, ambiguous requirements get silently guessed at during implementation — and the guess is usually wrong, so the work gets redone.

    -   **Solution:** the Spec step (see Workflow, below) forces every open question to be asked and locked as a numbered decision _before_ any code is written — this is where the **up to 80% rework reduction** comes from (early, informal figure).

-   **Rules get forgotten — by developers too, not just agents** — project conventions, coding standards, and hard rules (say, "no comments in code", "no warnings", "no `else`") live in docs nobody re-reads, so violations slip in during everyday work: a dev uses a naming pattern that was banned, adds a comment where the rule says none, or guesses at an edge-case convention that was already decided. The cost shows up later, in review time and rework.

    -   **Solution:** wiki-mem stores rules as first-class, active pages (`wiki:rules:*`) that every session loads at Init (see Workflow, below) — the same rules agents must obey are the same rules developers are reminded of at the start of each task, so conventions are applied up front instead of discovered mid-review.

-   **Savings are claims, not data — and the costliest lessons get paid for twice** — without time tracking, "AI has roughly halved our development time" is a claim nobody can verify; without Critical Patterns, the most expensive lessons (the tricky bug that cost a day, the deployment step that silently skipped) are re-learned from scratch by whoever hits them next.
    -   **Solution:** a built-in **timer** records real hours per task, turning the ~50% figure from anecdote into measured, auditable data — the same data the pilot will track formally; and **Critical Patterns** promotes the costliest lessons into a page loaded at the start of every session, so the compounding that makes the setup worthwhile is durable rather than depending on who happens to remember.

These figures are from our own early, informal experience and will be tracked more formally as we expand (see Proposal, below).

---

## 3. Workflow — How Wiki-mem Works

Every piece of work follows the same pipeline — each step writes something back into the wiki, so knowledge compounds as work proceeds.

### Step 1 — Steering

-   The starting point: the team decides what to work on (a task, a bug, a feature request)
-   Everything after this is driven by that direction

### Step 2 — Init

-   The knowledge base is bootstrapped for the session: current task state, active rules, and accumulated project memory are pulled together into a single session summary
-   Every session begins by loading the project's core documents: README (what the project is), Architecture (how it's built), Conventions (rules everyone follows), and Critical Patterns (the hardest-won lessons) — this is the "context" that makes AI work faster and correctly
-   What it writes to the wiki: nothing yet — it reads, so the session starts informed

### Step 3 — Spec

-   Before any coding, the work is specified: the AI identifies the open gray areas (scope, behaviour, edge cases, trade-offs) and asks the team one targeted question at a time
-   Every answer is **locked as a numbered decision** (D1, D2, D3…) inside the spec itself before moving to the next question — nothing is assumed or guessed
-   Once all gray areas are resolved, the locked decisions plus requirements, acceptance criteria, and scenarios are written into a spec page in the wiki, referenced by its own ID (`wiki:specs:<name>`), for review and approval
-   What it writes: a spec page with its locked decisions + approval status. (Standalone architecture-decision-record pages, noted by ID as `wiki:decisions:<name>`, are a separate artifact — see Extract, below.)
-   This is where the up to **80% rework reduction** comes from — every decision is made and confirmed with the team once, up front, instead of being assumed during coding and corrected afterward

### Step 4 — Plan

-   The spec is turned into a step-by-step implementation plan with testable tasks
-   What it writes: task pages with acceptance criteria, each noted by ID as `wiki:tasks:<name>`

### Step 5 — Implement

-   The work is executed following the plan
-   What it writes: task status updates, implementation notes, and any decisions made along the way

### Step 6 — Extract

-   Completed work is reviewed across three categories: reusable **patterns** (approaches worth repeating), **decisions** (good calls, bad calls, trade-offs — captured as standalone ADR pages noted by ID as `wiki:decisions:<name>`, distinct from the spec's inline locked decisions), and **failures** (bugs, wrong assumptions, wasted effort — captured so they aren't repeated)
-   Each captured page (`wiki:patterns:<name>`, `wiki:decisions:<name>`) is linked back to the source task ID it came from, so it stays discoverable through the wiki's graph instead of becoming a dead file
-   The most costly-to-learn lessons are promoted into the Critical Patterns page (`wiki:core:critical-patterns`) that every future session loads at Init (step 2) — this is the mechanism that makes the loop actually compound, not just accumulate
-   What it writes: pattern/decision/failure pages, graph edges back to the source task ID, and a short memory entry for fast recall

### Step 7 — Commit

-   The change is committed with wiki validation, so the code and the knowledge stay in sync

### Why the Loop Matters

The loop is the point: **every step writes something back**, so the next task — and the next person — starts further ahead than the last one did. No context is lost between sessions.

---

## 4. Guardrails — Kiro Hooks and Zero Trust

Giving agents write access and shell access raises an obvious question: what stops an agent from doing something destructive, whether by mistake or by being misled? Kiro's answer is **zero trust**: every agent's tool calls are checked by hooks before they're allowed to run — the agent is never simply trusted to behave, regardless of its instructions or how convincing a bad prompt might be.

Concretely, before any file write or shell command executes, it passes through a set of hard-coded guard scripts that block the call outright if it matches a dangerous pattern — not a warning, not a "please confirm," the call simply doesn't execute:

-   **Block dangerous commands** — destructive shell commands are blocked outright; the agent is told to stop and ask a human to run it, not to find a workaround.
-   **Block secrets in commands** — commands containing credential-shaped patterns are blocked, and any `git commit`/`git push` is checked against staged file contents first, so a secret can't slip into a commit.
-   **Block writes outside the workspace** — every file write and shell command is checked against the project's actual root (via `git rev-parse --show-toplevel`), so an agent cannot write outside the repository it was scoped to.
-   **Block unsafe JSON edits** — `sed`/`awk` in-place edits on JSON files are blocked in favour of a proper JSON tool, preventing silent file corruption.

This is enforced the same way for every agent, every time, regardless of which model is behind it or what the prompt says — that consistency is the point of zero trust: security doesn't depend on remembering to ask nicely.

**Honest limit:** these hooks are a guardrail against mistaken or subtly-misled tool calls, not a full sandbox — they're pattern-based, not a substitute for OS-level isolation (a container or a restricted user account with the repo as the only writable location) if the threat model requires it. We treat this as a known, accepted residual risk at the pilot's current scale rather than papering over it.

---

## Risks and Open Questions

Being upfront about what this brief doesn't yet answer:

-   **Single-team evidence.** The figures above come from one team's early use, not a controlled comparison — that's exactly why the pilot exists, and why we're proposing to measure rather than repeat the claim.
-   **Sending code to an external model provider.** Model inference sends session context — which can include our code — to the model provider (Claude/GPT). For a financial-services portal, this may need a compliance/security sign-off before wider rollout, even though the pilot itself is small and reversible.
-   **Cost isn't nailed down.** "Cut AI cost by 50%" is relative to our own prior usage, not an absolute dollar figure — per-seat API/licensing cost for the pilot group should be confirmed before scaling past it.
-   **Tooling dependency.** wiki-mem's data format and this workflow are specific to the current tooling; if that tooling stops being maintained, migrating the accumulated knowledge base is a real cost, not zero.
-   **What "success" and "failure" look like.** We haven't yet pre-defined a pass/fail threshold for the pilot's tracked metrics — we'll set one before the pilot starts, not after seeing the results, so the pilot has a clear stop condition if it doesn't deliver.

---

_Happy to walk through this in person. Questions welcome._
