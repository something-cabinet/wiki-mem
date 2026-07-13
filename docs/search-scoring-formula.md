# Search Scoring Formula

**Reference:** WM's BM25 implementation, field-weighted with rerank boosts, FSRS-6 recency, and RRF fusion.

## BM25 Score (per document field)

$$ \text{BM25}(D, Q) = \sum_{t \in Q} \text{IDF}(t) \cdot \frac{\text{tf}(t, D) \cdot (k_1 + 1)}{\text{tf}(t, D) + k_1 \left(1 - b + b \cdot \frac{|D|}{\text{avgdl}}\right)} \cdot w_f $$

$$ \text{IDF}(t) = 1 + \ln \frac{N - \text{df}(t) + 0.5}{\text{df}(t) + 0.5} $$

| Symbol | Meaning | Default |
|---|---|---|
| $N$ | Total documents in index | — |
| $\text{df}(t)$ | Documents containing term $t$ | — |
| $\text{tf}(t, D)$ | Frequency of term $t$ in document $D$ | — |
| $k_1$ | Term frequency saturation | 1.2 |
| $b$ | Length normalization | 0.75 |
| $\|D\|$ | Document length (tokens) | — |
| $\text{avgdl}$ | Average document length | — |
| $w_f$ | Field weight (title: 4.0, body: 1.0) | — |

> **Source:** Robertson & Zaragoza (2009), *The Probabilistic Relevance Framework: BM25 and Beyond*, Foundations and Trends in IR.

---

### Worked Example

**Index:** $N = 10,000$ documents, avgdl $= 150$ tokens  
**Query:** $Q = \{\text{neural}, \text{network}\}$

**Document A:** title (8 tokens), body (400 tokens). $\text{tf}(\text{neural}) = 3$, $\text{tf}(\text{network}) = 5$  
**Document B:** title (12 tokens), body (4,000 tokens). $\text{tf}(\text{neural}) = 6$, $\text{tf}(\text{network}) = 6$

**Step 1 — IDF:**
"neural" appears in 200 docs, "network" in 500 docs:

$$ \text{IDF}(\text{neural}) = 1 + \ln\frac{10000 - 200 + 0.5}{200 + 0.5} = 1 + \ln 48.88 = 4.89 $$

$$ \text{IDF}(\text{network}) = 1 + \ln\frac{10000 - 500 + 0.5}{500 + 0.5} = 1 + \ln 19.00 = 3.94 $$

**Step 2 — Title contribution (weight $w_f = 4.0$):**

For "neural" in Doc A's title ($|D| = 8$, $\text{tf}=1$):

$$ K = 1.2 \left(1 - 0.75 + 0.75 \cdot \frac{8}{150}\right) = 1.2 \cdot (0.25 + 0.04) = 0.348 $$

$$ \text{score} = 4.0 \cdot 4.89 \cdot \frac{1 \cdot 2.2}{1 + 0.348} = 4.0 \cdot 4.89 \cdot 1.632 = 31.92 $$

**Step 3 — Body contribution (weight $w_f = 1.0$):**

For "neural" in Doc A's body ($|D| = 400$, $\text{tf}=2$):

$$ K = 1.2 \left(0.25 + 0.75 \cdot \frac{400}{150}\right) = 1.2 \cdot (0.25 + 2.0) = 2.7 $$

$$ \text{score} = 1.0 \cdot 4.89 \cdot \frac{2 \cdot 2.2}{2 + 2.7} = 4.89 \cdot 0.936 = 4.58 $$

**Step 4 — Total BM25:**

| Term | Doc A title | Doc A body | Doc A total | Doc B total |
|---|---|---|---|---|
| "neural" | 31.92 | 4.58 | 36.50 | 7.24 |
| "network" | 18.52 | 3.12 | 21.64 | 5.89 |
| **Final** | | | **58.14** | **13.13** |

Doc A scores higher because it's shorter and more focused, despite having fewer absolute occurrences.

---

## Saturation Effect

The $k_1$ parameter controls how much each additional occurrence boosts the score. With $k_1 = 1.2$:

| TF | Raw contribution | Cumulative | % of max |
|---|---|---|---|
| 1 | 0.91 | 0.91 | 46% |
| 2 | 0.74 | 1.65 | 84% |
| 5 | 0.18 | 1.83 | 93% |
| 10 | 0.14 | 1.97 | 100% |

A 5th occurrence adds only 18% of the first. The 10th adds 14%. BM25 saturates — keyword stuffing doesn't work.

> Graph: $\frac{\text{tf} \cdot (k_1 + 1)}{\text{tf} + k_1}$ as a function of tf for $k_1 = 1.2$. Approaches asymptote $k_1 + 1 = 2.2$.

---

## Rerank Boosts (additive)

$$ \text{score} = \text{BM25} + \text{rerank\_boost} $$

| Condition | Boost |
|---|---|
| Title exactly matches query | +8.0 |
| Title starts with query | +4.0 |
| Title contains query | +2.0 |
| ID exactly matches query | +7.0 |
| Tag contains any query token | +3.0 |

### Worked Example

Query: "transformer architecture"

- Doc A: title = "Transformer Architecture for NLP" → exact match → **+8.0**
- Doc B: title = "Understanding Transformer Models" → contains match → **+2.0**
- Doc C: title = "CNN for Image Classification" → no match → **+0.0**

---

## Score Normalization

$$ \text{norm} = \max\left(0.01,\ \min\left(1.0,\ \frac{\text{round}(\text{score} / \text{max\_score} \times 10000)}{10000}\right)\right) $$

### Worked Example

Raw scores: [58.14, 13.13, 5.03], max = 58.14

| Raw | Normalized |
|---|---|
| 58.14 | 1.0000 |
| 13.13 | 0.2258 |
| 5.03 | 0.0865 |

The 0.01 floor ensures marginal matches still rank above non-matches.

---

## Memory Salience Boost

$$ \text{boost} = \min\left(\text{salience\_boost},\ \frac{\text{clamp}}{\text{score}}\right) $$
$$ \text{final} = \text{score} \times \text{boost} $$

Default: $\text{salience\_boost} = 2.0$, $\text{clamp} = 0.1$

### Worked Example

| BM25 score | $0.1 / \text{score}$ | Cap at 2.0 | Final |
|---|---|---|---|
| 0.03 | 3.33 | 2.0 | 0.06 |
| 0.08 | 1.25 | 1.25 | 0.10 |
| 0.15 | 0.67 | 0.67 | 0.10 |

Low-scoring entries get boosted more (up to 2×) to improve recall.

---

## Recency Boosts

### FSRS-6 Forgetting Curve (default)

$$ w_{20} = 0.1542 $$
$$ \text{factor} = 0.9^{-1/w_{20}} - 1 \approx 0.9^{-6.485} - 1 \approx 1.828 $$
$$ r = \left(1 + \text{factor} \cdot \frac{d}{s}\right)^{-w_{20}} $$
$$ \text{recency} = \text{clamp}(r, 0, 1) $$

Default stability $s = 7$ days. Applied as: $\text{score} = \text{score} \times \text{recency}$

### Worked Example

| Days since update ($d$) | FSRS-6 recency | Linear recency | Exponential recency |
|---|---|---|---|
| 0 (today) | 1.000 | 1.000 | 1.000 |
| 1 | 0.953 | 0.857 | 0.867 |
| 3 | 0.869 | 0.571 | 0.651 |
| 7 | 0.746 | 0.000 | 0.368 |
| 14 | 0.566 | 0.000 | 0.135 |
| 30 | 0.366 | 0.000 | 0.014 |

A task updated 30 days ago scores 63% lower than a fresh one.

---

## RRF Fusion (Hybrid Search)

$$ \text{RRF}(id) = \sum_{t \in T} \frac{1}{k + \text{rank}_t(id)} $$

Where $k = 60$, $T = \{\text{keyword}, \text{semantic}\}$ (or $\{\text{pages}, \text{memory}\}$).

### Worked Example

| ID | Keyword rank | Semantic rank | RRF score |
|---|---|---|---|
| doc-1 | 1 | 3 | $1/61 + 1/63 = 0.0323$ |
| doc-2 | 2 | 1 | $1/62 + 1/61 = 0.0325$ |
| doc-3 | — | 2 | $0 + 1/62 = 0.0161$ |

Doc-2 ranks first despite being rank 2 in keyword, because semantic ranks it highest.

---

## Total Score Pipeline

### Keyword path (pages)

$$
\begin{aligned}
\text{raw}    &= \text{BM25}(D, Q)                &&\to [0, \infty)  \\
\text{norm}   &= \text{normalize}(\text{raw})      &&\to [0, 1]      \\
\text{boosted}&= \text{norm} + \text{rerank\_boost} &&\to [0, \sim 12] \\
\text{final}  &= \text{normalize}([\text{boosted}]) &&\to [0.01, 1.0]
\end{aligned}
$$

### Semantic path (pages)

$$ \text{cosine}(q, v) = \frac{q \cdot v}{\|q\| \|v\|} \to [0, 1] $$

### Hybrid path (pages)

$$ \text{RRF}(id) = \frac{1}{k + \text{rank}_{\text{kw}}(id)} + \frac{1}{k + \text{rank}_{\text{sem}}(id)} $$

### Memory path (keyword)

$$
\begin{aligned}
\text{raw}   &= \text{BM25}(M, Q)              &&\to [0, \infty)  \\
\text{norm}  &= \text{normalize}(\text{raw})    &&\to [0.01, 1.0] \\
\text{boost} &= \min(2.0,\ 0.1 / \text{norm})  \\
\text{final} &= \text{norm} \times \text{boost} &&\to [0.01, 2.0]
\end{aligned}
$$

### Cross-type fusion

$$ \text{RRF}(id) = \frac{1}{k + \text{rank}_{\text{pages}}(id)} + \frac{1}{k + \text{rank}_{\text{memory}}(id)} $$

### Final sort

Results sorted by:
1. Score descending
2. Centrality (inbound graph edges, **weighted by edge type priority**) descending
3. Page type rank descending
4. Title ascending

Centrality is not a raw edge count — each inbound edge contributes its type's priority:
$$ \text{centrality} = \sum_{e \in \text{inbound}} \text{priority}(\text{type}(e)) $$

| Edge | Priority | Contribution |
|---|---|---|
| `extends` | 10 | 10 per edge |
| `implements` | 9 | 9 per edge |
| `relates_to` | 0 | 0 per edge (structural link, no boost) |

A page with 5 `extends` edges (score 50) outranks a page with 10 `relates_to` edges (score 0).

| Page Type | Priority | Semantic | Example |
|-----------|----------|----------|---------|
| `task` | 7 | Actionable work unit | `fix-auth-timeout` |
| `spec` | 6 | Requirements specification | `user-auth` |
| `pattern` | 5 | Reusable solution | `arc-swap-graph` |
| `concept` | 4 | Domain explanation | `bm25-search` |
| `decision` | 3 | ADR lifecycle record | `axum-over-rocket` |
| `howto` | 2 | Step-by-step guide | `platform-setup` |
| `reference` | 1 | API/config reference | `scoring-config` |
| `note` | 0 | Informal content | `meeting-notes` |

---

## Constants Summary

| Constant | Value | Role |
|---|---|---|
| $k_1$ | 1.2 | Term frequency saturation |
| $b$ | 0.75 | Length normalization |
| $w_{20}$ | 0.1542 | FSRS-6 forgetting curve |
| RRF $k$ | 60 | Fusion sharpness |
| Title weight | 4.0 | Field weight |
| Body weight | 1.0 | Field weight |
| Stability days | 7 | Recency half-life |
| Salience boost | 2.0 | Memory ceiling |
| Salience clamp | 0.1 | Memory threshold |
