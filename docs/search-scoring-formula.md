# Search Scoring Formula

## BM25 Score (per document field)

For each document field, the BM25 score is:

$$ \text{BM25}(doc, query) = \sum_{field} \text{field\_score} \times \text{boost} $$

$$ \text{field\_score} = w \times \text{idf} \times \frac{tf \times (k_1 + 1)}{tf + k_1 \times (1 - b + b \times \frac{fl}{al})} $$

$$ \text{idf} = 1 + \ln\frac{N - df + 0.5}{df + 0.5} $$

Where:
- $tf$ = term frequency in field
- $df$ = document frequency (docs containing term)
- $N$ = total documents in index
- $k_1 = 1.2$ (BM25_K1)
- $b = 0.75$ (BM25_B)
- $w$ = field weight (title: 4.0, body: 1.0)
- $fl$ = field length (tokens in this field)
- $al$ = average field length across all docs

## Rerank Boosts (additive)

$$ \text{final} = \text{BM25\_score} + \text{rerank\_boost} $$

| Condition | Boost |
|---|---|
| Title exactly matches query | +8.0 |
| Title starts with query | +4.0 |
| Title contains query | +2.0 |
| ID exactly matches query | +7.0 |
| Tag contains any query token | +3.0 |

## Score Normalization

After BM25 scoring, all scores are normalized to $[0.01, 1.0]$:

$$ \text{normalized} = \max\left(0.01,\ \min\left(1.0,\ \frac{\lfloor \frac{\text{score}}{\text{max\_score}} \times 10000 \rceil}{10000}\right)\right) $$

The $0.01$ floor ensures partial matches rank above non-matches.

## Memory Salience Boost

$$ \text{boost} = \min(\text{salience\_boost},\ \frac{\text{clamp}}{\text{score}}) $$
$$ \text{final} = \text{score} \times \text{boost} $$

Default: $\text{salience\_boost} = 2.0$, $\text{clamp} = 0.1$

## Recency Boosts

### FSRS-6 Forgetting Curve (default)

$$ w_{20} = 0.1542 $$
$$ \text{factor} = 0.9^{-1/w_{20}} - 1 $$
$$ r = \left(1 + \text{factor} \times \frac{d}{s}\right)^{-w_{20}} $$
$$ \text{recency} = \text{clamp}(r, 0, 1) $$

Default $s = 7\ \text{days}$. Applied as: $\text{score} = \text{score} \times \text{recency}$

### Linear

$$ \text{recency} = \max\left(0,\ 1 - \frac{d}{s}\right) $$

### Exponential

$$ \text{recency} = e^{-d/s} $$

## Hybrid Search (RRF Fusion)

$$ \text{RRF}(id) = \sum_{t \in types} \frac{1}{k + \text{rank}_t(id)} $$

Where $k = 60$ (rrf_k) and $types$ = {keyword, semantic, page, memory}.

## Search Mode

| Mode | Page results | Memory results | Fusion |
|---|---|---|---|
| `keyword` | BM25 + rerank | BM25 (memory index) | RRF |
| `semantic` | Cosine similarity | Cosine similarity | RRF |
| `hybrid` | BM25 + cosine via RRF | BM25 + cosine via score merge | RRF |

## Total Score Pipeline

### Keyword path (pages)

$$ \begin{aligned}
\text{raw} &= \text{BM25}(doc, query)      &&\to [0, \infty) \\
\text{norm} &= \text{normalize}(\text{raw}) &&\to [0, 1] \\
\text{boosted} &= \text{norm} + \text{rerank\_boost} &&\to [0, \sim 12] \\
\text{final} &= \text{normalize}([\text{boosted}]) &&\to [0.01, 1.0]
\end{aligned} $$

### Semantic path (pages)

$$ \text{cosine}(q, d) = \frac{q \cdot d}{\|q\| \|d\|} \to [0, 1] $$

### Hybrid path (pages)

$$ \text{RRF}(id) = \frac{1}{k + \text{rank}_{\text{keyword}}(id)} + \frac{1}{k + \text{rank}_{\text{semantic}}(id)} $$

### Memory path (keyword)

$$ \begin{aligned}
\text{raw} &= \text{BM25}(entry, query)    &&\to [0, \infty) \\
\text{norm} &= \text{normalize}(\text{raw}) &&\to [0.01, 1.0] \\
\text{salience} &= \min(\text{boost}, \frac{\text{clamp}}{\text{norm}}) \\
\text{final} &= \text{norm} \times \text{salience} &&\to [0.01, 2.0]
\end{aligned} $$

### Cross-type fusion

$$ \text{RRF}(id) = \frac{1}{k + \text{rank}_{\text{pages}}(id)} + \frac{1}{k + \text{rank}_{\text{memory}}(id)} $$

### Final sort

Results sorted by:
1. Score descending
2. Centrality (inbound graph edges) descending
3. Page type rank descending
4. Title ascending

Where page type ranks:
$$ \text{task}=7 \to \text{spec}=6 \to \text{pattern}=5 \to \text{concept}=4 \to \text{decision}=3 \to \text{howto}=2 \to \text{reference}=1 \to \text{note}=0 $$

## Constants Summary

| Constant | Value |
|---|---|
| $k_1$ | 1.2 |
| $b$ | 0.75 |
| $w_{20}$ | 0.1542 |
| RRF $k$ | 60 |
| Title weight | 4.0 |
| Body weight | 1.0 |
| Recency model | "fsrs" |
| Stability days | 7 |
| Salience boost | 2.0 |
| Salience clamp | 0.1 |
