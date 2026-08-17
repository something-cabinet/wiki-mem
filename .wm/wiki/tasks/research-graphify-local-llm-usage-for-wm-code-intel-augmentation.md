---
title: Research Graphify local LLM usage for wm code-intel augmentation
type: task
id: "wiki:tasks:research-graphify-local-llm-usage-for-wm-code-intel-augmentation"
status: todo
priority: medium
tags: [research, graphify, llm, ollama, code-intel]
acceptance_criteria:
  - text: "Document how Graphify llm.py multi-backend architecture works: backend registry (claude/kimi/ollama/gemini/openai/deepseek/azure/bedrock), OpenAI-compat adapter pattern, tiktoken token counting, file-slice packing with char cap, and the --backend flag dispatch"
  - text: "Evaluate which patterns could benefit wm code-intel extraction: LLM-augmented INFERRED edges (call-graph second pass), community labelling, or graph enrichment where tree-sitter alone is insufficient"
  - text: "Assess feasibility of adding an optional Ollama backend to wm for local LLM-augmented extraction (qwen2.5-coder default), with graceful degradation when unavailable"
  - text: "Document the comparison: Graphify uses LLM as an extraction augmenter (tree-sitter first, LLM for ambiguous/inferred edges); wiki-mem uses ONNX embeddings for search but no LLM for code-intel extraction"
---

Investigate Graphify's local LLM integration for potential adoption in wiki-mem.

Graphify (github.com/Graphify-Labs/graphify v8) has a multi-backend LLM module (graphify/llm.py) supporting:
- Ollama (local, default qwen2.5-coder:7b, zero cost, OpenAI-compat /v1 endpoint)
- Kimi (Moonshot kimi-k2.6, multimodal)
- Gemini (Google, OpenAI-compat endpoint)
- OpenAI (gpt-4.1-mini default, supports llama.cpp/vLLM/LM Studio via OPENAI_BASE_URL)
- DeepSeek (v4-flash, thinking-enabled)
- Azure OpenAI
- Bedrock (Anthropic Claude via boto3)
- Claude (Anthropic direct, claude-sonnet-4-6 default)

Key patterns:
1. All backends use the OpenAI client library (except Bedrock/Azure with their own SDKs)
2. Token counting via tiktoken cl100k_base (fallback: 4 chars/token heuristic)
3. File slicing with 20K char cap per file, concurrent extraction via ThreadPoolExecutor
4. Used for extraction augmentation (tree-sitter does the heavy lifting; LLM handles ambiguous/inferred edges and community labelling)
5. Graceful degradation: tree-sitter extraction works without any LLM; LLM is opt-in via --backend flag

wiki-mem currently has ONNX embeddings (bge-small) for semantic search but NO LLM integration for code-intel extraction. The question is whether adding an optional local-LLM augmentation layer (Ollama) would improve edge quality for ambiguous resolution cases where tree-sitter alone produces AMBIGUOUS provenance.