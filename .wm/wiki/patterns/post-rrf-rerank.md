---
id: wiki:patterns:post-rrf-rerank
---
id: wiki:patterns:post-rrf-rerank

Move rerank boosts to AFTER RRF fusion as a separate post-processing step. Apply Knowns-inspired heuristics on the fused scores where boosts actually take effect:

- Title density: +0.03 per query token found in title
- Exact title match: +0.15 additive (raw query, not stemmed)
- Title starts with query: +0.08 additive (raw query)
- Title contains query: +0.04 additive (raw query)
- Tag overlap: proportional (matched/total × 0.1 × score)
- Exact ID match: +0.10 additive (raw query)

The starts_with (+0.08) and contains (+0.04) equivalents were added 2026-07-24, mirroring the keyword mode boosts (+4.0 and +2.0) at hybrid-appropriate scale. All phrase-level comparisons use the raw (un-stemmed) query string so Snowball stemming doesn't disable them.