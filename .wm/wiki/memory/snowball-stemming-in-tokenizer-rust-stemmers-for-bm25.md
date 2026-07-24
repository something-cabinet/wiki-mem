---
title: Snowball stemming in tokenizer — rust-stemmers for BM25
type: memory
tags: [search, bm25, tokenizer, stemming]
status: active
---

Tokenizer uses rust-stemmers (Snowball English, Porter2) to normalize morphological variants. "patterns"→"pattern", "designer"→"design", "styling"→"style". Stemmed form is additional (original kept). Uses LazyLock, shared across rayon threads. Full reference: @wiki/reference:search-scoring-formula